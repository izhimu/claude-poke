use crate::animation::{AnimationMap, FrameManager, SpriteSheet};
use crate::animation::sprite_sheet::create_placeholder_sprite;
use crate::config::{get_assets_dir, get_http_port, WindowConfig, SPRITE_SIZE};
use crate::monitor::HttpServer;
#[cfg(target_os = "windows")]
use crate::render::GdiRenderer;
#[cfg(not(target_os = "windows"))]
use crate::render::PetRenderer;
use crate::state::{StateMachine, StatusFile};
#[cfg(all(feature = "tray", target_os = "linux"))]
use crate::system::tray::{SystemTray, MENU_ALWAYS_ON_TOP, MENU_QUIT, MENU_SHOW_HIDE};

use anyhow::Result;
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId, WindowLevel};

/// User events for the event loop.
/// Currently unused — tray events flow through mpsc channel instead.
/// Kept as a type parameter for EventLoop::<UserEvent>::with_user_event().
#[derive(Debug)]
pub enum UserEvent {}

/// Type alias for the platform-specific renderer.
#[cfg(target_os = "windows")]
type Renderer = GdiRenderer;
#[cfg(not(target_os = "windows"))]
type Renderer = PetRenderer;

/// The main application state.
pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    state_machine: StateMachine,
    frame_manager: FrameManager,
    animation_map: AnimationMap,
    config: WindowConfig,
    assets_dir: PathBuf,
    status_rx: Option<mpsc::Receiver<StatusFile>>,
    _http_server: Option<HttpServer>,
    visible: bool,
    always_on_top: bool,
    #[cfg(all(feature = "tray", target_os = "linux"))]
    tray: Option<SystemTray>,
    /// Channel to send frame data to the Wayland layer surface render thread.
    #[cfg(target_os = "linux")]
    layer_tx: Option<mpsc::Sender<Option<Vec<u8>>>>,
    /// Whether we're running on Wayland (layer surface mode).
    #[cfg(target_os = "linux")]
    is_wayland: bool,
}

impl App {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

        // Initialize system tray (feature-gated, Linux-only)
        #[cfg(all(feature = "tray", target_os = "linux"))]
        let tray = match SystemTray::new() {
            Ok(t) => {
                info!("System tray initialized");
                Some(t)
            }
            Err(e) => {
                warn!("Failed to create system tray: {}", e);
                None
            }
        };

        Self {
            window: None,
            renderer: None,
            state_machine: StateMachine::new(),
            frame_manager: FrameManager::new(1, 1.0), // frame count auto-detected from sprite sheet
            animation_map: AnimationMap::new(),
            config: WindowConfig::default(),
            assets_dir: get_assets_dir(),
            status_rx: None,
            _http_server: None,
            visible: true,
            always_on_top: true,
            #[cfg(all(feature = "tray", target_os = "linux"))]
            tray,
            #[cfg(target_os = "linux")]
            layer_tx: None,
            #[cfg(target_os = "linux")]
            is_wayland,
        }
    }

    /// Start the HTTP status server.
    pub fn start_monitoring(&mut self) -> Result<()> {
        let port = get_http_port();
        let (tx, rx) = mpsc::channel();
        let server = HttpServer::start(port, tx)?;
        info!("Status server started — hooks should POST to http://127.0.0.1:{}/status", port);
        self.status_rx = Some(rx);
        self._http_server = Some(server);
        Ok(())
    }

    /// Build a single frame as raw RGBA pixel data.
    fn build_frame(&mut self) -> Vec<u8> {
        let state = self.state_machine.current().clone();
        let anim_def = self.animation_map.get(&state);
        let frame_idx = self.frame_manager.current_frame();
        let win_w = self.config.width;
        let win_h = self.config.height;

        // Try to load sprite sheet, fall back to placeholder
        let sprite_path = self
            .assets_dir
            .join("sprites")
            .join(format!("{}.png", anim_def.sprite_name));

        if sprite_path.exists() {
            // Load at native sprite resolution, then upscale to window size
            match SpriteSheet::load(&sprite_path, SPRITE_SIZE, SPRITE_SIZE) {
                Ok(sheet) => {
                    // Auto-detect frame count from sprite sheet and update FrameManager
                    let detected = sheet.frame_count();
                    if self.frame_manager.frame_count() != detected {
                        self.frame_manager
                            .set_animation(detected, anim_def.fps);
                    }
                    let native = sheet.frame_data(frame_idx);
                    return nearest_neighbor_scale(&native, SPRITE_SIZE, SPRITE_SIZE, win_w, win_h);
                }
                Err(e) => {
                    warn!("Failed to load sprite {}: {}", sprite_path.display(), e);
                }
            }
        }

        // Placeholder: generate at window size directly
        let fc = self.frame_manager.frame_count().max(1);
        let brightness_mod = ((frame_idx as f32 / fc as f32) * 20.0) as u8;
        let mut color = anim_def.color;
        color[0] = color[0].saturating_add(brightness_mod);
        color[1] = color[1].saturating_add(brightness_mod);
        color[2] = color[2].saturating_add(brightness_mod);
        create_placeholder_sprite(win_w, win_h, color)
    }

    /// Render a single frame using the pixels renderer (X11 path).
    fn render_frame(&mut self) {
        // Build frame data first to avoid borrow conflict with renderer
        let sprite_data = self.build_frame();

        let renderer = match &mut self.renderer {
            Some(r) => r,
            None => return,
        };

        renderer.clear();
        renderer.draw_sprite(&sprite_data, self.config.width, self.config.height, 0, 0);

        if let Err(e) = renderer.render() {
            error!("Render error: {}", e);
        }
    }

    /// Send frame data to the Wayland layer surface render thread.
    #[cfg(target_os = "linux")]
    fn send_frame_to_layer(&mut self) {
        let sprite_data = self.build_frame();
        if let Some(ref tx) = self.layer_tx {
            if tx.send(Some(sprite_data)).is_err() {
                warn!("Layer surface render thread disconnected");
                self.layer_tx = None;
            }
        }
    }

    /// Check for status updates from the HTTP server.
    fn check_status_updates(&mut self, _event_loop: &ActiveEventLoop) {
        let rx = match &self.status_rx {
            Some(rx) => rx,
            None => return,
        };

        // Drain all pending updates, keep the latest
        let mut latest = None;
        while let Ok(status) = rx.try_recv() {
            latest = Some(status);
        }

        if let Some(status) = latest {
            debug!("Status update: {:?}", status.state);
            if self.state_machine.transition(status.state.clone()) {
                let new_state = self.state_machine.current().clone();
                let anim_def = self.animation_map.get(&new_state);
                // Frame count is auto-detected from the sprite sheet in build_frame
                self.frame_manager.set_animation(0, anim_def.fps);
                info!("State changed to: {}", new_state);
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        if self.is_wayland && self.layer_tx.is_none() && self.window.is_none() {
            // On Wayland: try layer surface renderer for always-on-top
            if crate::render::wayland_layer::is_layer_shell_available() {
                match crate::render::wayland_layer::spawn_layer_renderer(&self.config) {
                    Ok(tx) => {
                        self.layer_tx = Some(tx);
                        info!("Wayland layer surface renderer spawned (always-on-top)");
                        return;
                    }
                    Err(e) => {
                        warn!("Failed to spawn layer renderer: {}", e);
                    }
                }
            } else {
                info!("Layer shell not available, using regular window");
            }
            // Fall through to create a regular winit window
        }

        #[cfg(target_os = "linux")]
        if self.layer_tx.is_some() {
            return;
        }

        if self.window.is_some() {
            return;
        }

        let (logical_w, logical_h) = self.config.logical_size();

        let attrs = crate::render::window::pet_window_attributes(
            "Claude Poke",
            logical_w,
            logical_h,
            self.always_on_top,
        );

        match event_loop.create_window(attrs) {
            Ok(window) => {
                // Position window at bottom-right of screen (using physical coordinates)
                if let Some(monitor) = window.current_monitor() {
                    let scale = monitor.scale_factor();
                    let screen_size = monitor.size();
                    let margin = 20;
                    let phys_w = (logical_w as f64 * scale) as u32;
                    let phys_h = (logical_h as f64 * scale) as u32;
                    let x = screen_size.width as i32 - phys_w as i32 - margin;
                    let y = screen_size.height as i32 - phys_h as i32 - margin;
                    window.set_outer_position(PhysicalPosition::new(x, y));
                }

                match Renderer::new(&window, &self.config) {
                    Ok(renderer) => {
                        self.renderer = Some(renderer);
                        info!("Renderer initialized");
                    }
                    Err(e) => {
                        error!("Failed to create renderer: {}", e);
                    }
                }

                self.window = Some(window);
                info!("Window created (logical {}x{})", logical_w, logical_h);
            }
            Err(e) => {
                error!("Failed to create window: {}", e);
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        // UserEvent is currently an empty enum — this match is unreachable.
        match event {}
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                self.window = None;
                self.renderer = None;
                self.visible = false;
                #[cfg(all(feature = "tray", target_os = "linux"))]
                if let Some(ref tray) = self.tray {
                    tray.set_visibility_label(false);
                }
                #[cfg(target_os = "linux")]
                if let Some(tx) = self.layer_tx.take() {
                    let _ = tx.send(None);
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    // Use the OS/WM native drag protocol.
                    // Works on both X11 (_NET_WM_MOVERESIZE) and Wayland (xdg_toplevel_move).
                    if let Some(window) = &self.window {
                        if let Err(e) = window.drag_window() {
                            debug!("drag_window failed: {}", e);
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.resize_surface(size.width, size.height) {
                        error!("Resize error: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check for status updates and apply deferred state transitions
        self.check_status_updates(event_loop);
        self.state_machine.tick();
        self.frame_manager.update();

        // Poll system tray menu events (drain all pending events per frame)
        #[cfg(all(feature = "tray", target_os = "linux"))]
        if let Some(ref tray) = self.tray {
            while let Some(id) = tray.try_recv_menu_event() {
                match id.as_str() {
                    MENU_SHOW_HIDE => {
                        self.visible = !self.visible;
                        if let Some(window) = &self.window {
                            window.set_visible(self.visible);
                            if self.visible && self.always_on_top {
                                window.set_window_level(WindowLevel::AlwaysOnTop);
                            }
                        }
                        tray.set_visibility_label(self.visible);
                    }
                    MENU_ALWAYS_ON_TOP => {
                        self.always_on_top = !self.always_on_top;
                        if let Some(window) = &self.window {
                            if self.always_on_top {
                                window.set_window_level(WindowLevel::AlwaysOnTop);
                            } else {
                                window.set_window_level(WindowLevel::Normal);
                            }
                        }
                        tray.set_always_on_top(self.always_on_top);
                    }
                    MENU_QUIT => {
                        info!("Quit requested from tray");
                        self.window = None;
                        self.renderer = None;
                        if let Some(tx) = self.layer_tx.take() {
                            let _ = tx.send(None);
                        }
                        event_loop.exit();
                        return;
                    }
                    _ => {}
                }
            }
        }

        // Sleep until the next animation frame is due, avoiding busy-wait.
        let next_frame = self.frame_manager.next_frame_deadline();
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));

        #[cfg(target_os = "linux")]
        if self.layer_tx.is_some() {
            // On Wayland with layer surface: send frame data to render thread
            self.send_frame_to_layer();
            return;
        }

        // On X11 (or Wayland fallback): request redraw for the winit window
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Nearest-neighbor upscale RGBA pixel data (preserves pixel-art crispness).
fn nearest_neighbor_scale(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    for y in 0..dst_h {
        let src_y = (y * src_h / dst_h) as usize;
        for x in 0..dst_w {
            let src_x = (x * src_w / dst_w) as usize;
            let si = (src_y * src_w as usize + src_x) * 4;
            let di = (y * dst_w + x) as usize * 4;
            dst[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    dst
}

/// Run the application.
pub fn run() -> Result<()> {
    // On Wayland: try layer-shell first, fall back to XWayland for always-on-top
    #[cfg(target_os = "linux")]
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if crate::render::wayland_layer::is_layer_shell_available() {
            info!("Layer shell available, using native Wayland");
        } else if std::env::var("DISPLAY").is_ok() {
            // GNOME doesn't support layer-shell, but supports XWayland.
            // Unset WAYLAND_DISPLAY to force winit to use X11, where
            // WindowLevel::AlwaysOnTop works via _NET_WM_STATE_ABOVE.
            info!("Layer shell not available, falling back to XWayland for always-on-top");
            std::env::remove_var("WAYLAND_DISPLAY");
        } else {
            warn!("No layer shell and no XWayland (DISPLAY), always-on-top will not work");
        }
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let mut app = App::new();
    app.start_monitoring()?;

    event_loop.run_app(&mut app)?;
    Ok(())
}
