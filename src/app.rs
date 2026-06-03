use crate::animation::{AnimationMap, FrameManager, SpriteSheet};
use crate::animation::sprite_sheet::create_placeholder_sprite;
use crate::config::{get_assets_dir, get_status_file_path, WindowConfig};
use crate::monitor::{FileWatcher, StatusPoller};
use crate::render::{DragState, PetRenderer};
use crate::state::{PetState, StateMachine, StatusFile};

use anyhow::Result;
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// User events for the event loop (from tray, etc.)
#[derive(Debug)]
#[allow(dead_code)]
pub enum UserEvent {
    StateChange(PetState),
    TrayShowHide,
    TrayQuit,
}

/// The main application state.
#[allow(dead_code)]
pub struct App {
    window: Option<Window>,
    renderer: Option<PetRenderer>,
    state_machine: StateMachine,
    frame_manager: FrameManager,
    animation_map: AnimationMap,
    drag_state: DragState,
    config: WindowConfig,
    assets_dir: PathBuf,
    status_rx: Option<mpsc::Receiver<Option<StatusFile>>>,
    _file_watcher: Option<FileWatcher>,
    last_state: PetState,
    visible: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            state_machine: StateMachine::new(),
            frame_manager: FrameManager::new(6, 1.0),
            animation_map: AnimationMap::new(),
            drag_state: DragState::new(),
            config: WindowConfig::default(),
            assets_dir: get_assets_dir(),
            status_rx: None,
            _file_watcher: None,
            last_state: PetState::Sleeping,
            visible: true,
        }
    }

    /// Initialize the file watcher in a background thread.
    pub fn start_monitoring(&mut self) -> Result<()> {
        let status_path = get_status_file_path();
        let mut file_watcher = FileWatcher::new(status_path.clone())?;
        file_watcher.start()?;

        let (tx, rx) = mpsc::channel();

        // Spawn a thread that reads from the file watcher and sends to the app
        std::thread::spawn(move || {
            let mut poller = StatusPoller::new(status_path.clone(), 100);
            let mut file_watcher_for_thread = FileWatcher::new(status_path).unwrap();
            let _ = file_watcher_for_thread.start();

            loop {
                // Try file watcher first
                if let Some(status) = file_watcher_for_thread.try_recv_status() {
                    let _ = tx.send(Some(status));
                } else {
                    // Fall back to polling
                    if let Some(status) = poller.poll() {
                        let _ = tx.send(Some(status));
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        self.status_rx = Some(rx);
        self._file_watcher = Some(file_watcher);
        Ok(())
    }

    /// Render a single frame using placeholder graphics.
    fn render_frame(&mut self) {
        let renderer = match &mut self.renderer {
            Some(r) => r,
            None => return,
        };

        renderer.clear();

        let state = self.state_machine.current().clone();
        let anim_def = self.animation_map.get(&state);
        let frame_idx = self.frame_manager.current_frame();

        // Try to load sprite sheet, fall back to placeholder
        let sprite_path = self
            .assets_dir
            .join("sprites")
            .join(format!("{}.png", anim_def.sprite_name));

        let sprite_data = if sprite_path.exists() {
            match SpriteSheet::load(&sprite_path, self.config.width, self.config.height) {
                Ok(sheet) => {
                    sheet.frame_data(frame_idx)
                }
                Err(e) => {
                    warn!("Failed to load sprite {}: {}", sprite_path.display(), e);
                    create_placeholder_sprite(self.config.width, self.config.height, anim_def.color)
                }
            }
        } else {
            // Use placeholder with animated brightness based on frame
            let brightness_mod = ((frame_idx as f32 / anim_def.frame_count as f32) * 20.0) as u8;
            let mut color = anim_def.color;
            color[0] = color[0].saturating_add(brightness_mod);
            color[1] = color[1].saturating_add(brightness_mod);
            color[2] = color[2].saturating_add(brightness_mod);
            create_placeholder_sprite(self.config.width, self.config.height, color)
        };

        renderer.draw_sprite(&sprite_data, self.config.width, self.config.height, 0, 0);

        if let Err(e) = renderer.render() {
            error!("Render error: {}", e);
        }
    }

    /// Check for status updates from the file watcher.
    fn check_status_updates(&mut self, _event_loop: &ActiveEventLoop) {
        let rx = match &self.status_rx {
            Some(rx) => rx,
            None => return,
        };

        // Drain all pending updates, keep the latest
        let mut latest = None;
        while let Ok(status) = rx.try_recv() {
            latest = status;
        }

        if let Some(status) = latest {
            debug!("Status update: {:?}", status.state);
            if self.state_machine.transition(status.state.clone()) {
                let new_state = self.state_machine.current().clone();
                let anim_def = self.animation_map.get(&new_state);
                self.frame_manager
                    .set_animation(anim_def.frame_count, anim_def.fps);
                info!("State changed to: {}", new_state);
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (physical_w, physical_h) = self.config.physical_size();

        let attrs = crate::render::window::pet_window_attributes(
            "Claude Poke",
            physical_w,
            physical_h,
        );

        match event_loop.create_window(attrs) {
            Ok(window) => {
                // Position window at bottom-right of screen
                if let Some(monitor) = window.current_monitor() {
                    let screen_size = monitor.size();
                    let margin = 20;
                    let x = screen_size.width as i32 - physical_w as i32 - margin;
                    let y = screen_size.height as i32 - physical_h as i32 - margin;
                    window.set_outer_position(PhysicalPosition::new(x, y));
                }

                match PetRenderer::new(&window, &self.config) {
                    Ok(renderer) => {
                        self.renderer = Some(renderer);
                        info!("Renderer initialized");
                    }
                    Err(e) => {
                        error!("Failed to create renderer: {}", e);
                    }
                }

                self.window = Some(window);
                info!("Window created ({}x{})", physical_w, physical_h);
            }
            Err(e) => {
                error!("Failed to create window: {}", e);
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::StateChange(state) => {
                if self.state_machine.transition(state.clone()) {
                    let anim_def = self.animation_map.get(&state);
                    self.frame_manager
                        .set_animation(anim_def.frame_count, anim_def.fps);
                    info!("State changed to: {}", state);
                }
            }
            UserEvent::TrayShowHide => {
                self.visible = !self.visible;
                if let Some(window) = &self.window {
                    window.set_visible(self.visible);
                }
            }
            UserEvent::TrayQuit => {
                info!("Quit requested");
                self.window = None;
                self.renderer = None;
            }
        }
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
            }
            WindowEvent::RedrawRequested => {
                self.check_status_updates(event_loop);
                self.frame_manager.update();
                self.render_frame();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            if let Some(window) = &self.window {
                                let window_pos = window
                                    .outer_position()
                                    .unwrap_or(PhysicalPosition::new(0, 0));
                                self.drag_state.start_drag(window_pos);
                            }
                        }
                        ElementState::Released => {
                            self.drag_state.end_drag();
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(new_pos) = self.drag_state.on_cursor_moved(position) {
                    if let Some(window) = &self.window {
                        window.set_outer_position(new_pos);
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request continuous redraws for animation
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Run the application.
pub fn run() -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let mut app = App::new();
    app.start_monitoring()?;

    event_loop.run_app(&mut app)?;
    Ok(())
}
