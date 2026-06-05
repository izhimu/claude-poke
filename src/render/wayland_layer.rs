use super::wayland_window::WaylandWindow;
use crate::config::WindowConfig;
use anyhow::Result;
use log::{debug, error, info, warn};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::mpsc;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, QueueHandle,
};

/// Frame data sent from the main thread to the render thread.
pub type FrameData = Vec<u8>;

/// Check if the wlr-layer-shell protocol is available on the current compositor.
pub fn is_layer_shell_available() -> bool {
    let Ok(conn) = Connection::connect_to_env() else {
        return false;
    };

    // Create a temporary event queue to enumerate globals
    let Ok((globals, _event_queue)) = registry_queue_init::<RegistryOnly>(&conn) else {
        return false;
    };

    globals.contents().with_list(|list| {
        list.iter().any(|g| g.interface == "zwlr_layer_shell_v1")
    })
}

/// Minimal state for registry-only operations.
struct RegistryOnly;
smithay_client_toolkit::delegate_registry!(RegistryOnly);
impl ProvidesRegistryState for RegistryOnly {
    fn registry(&mut self) -> &mut RegistryState {
        unreachable!()
    }
    smithay_client_toolkit::registry_handlers![];
}

/// Entry point: spawn a layer surface render thread.
/// Returns a channel sender for pushing frame data.
pub fn spawn_layer_renderer(config: &WindowConfig) -> Result<mpsc::Sender<Option<FrameData>>> {
    let (tx, rx) = mpsc::channel::<Option<FrameData>>();
    let width = config.width;
    let height = config.height;

    std::thread::Builder::new()
        .name("wayland-layer".into())
        .spawn(move || {
            if let Err(e) = run_layer_surface(width, height, rx) {
                error!("Layer surface render thread error: {}", e);
            }
        })?;

    Ok(tx)
}

/// Internal: run the layer surface event loop on a dedicated thread.
fn run_layer_surface(
    logical_w: u32,
    logical_h: u32,
    frame_rx: mpsc::Receiver<Option<FrameData>>,
) -> Result<()> {
    // Connect to the Wayland compositor
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<LayerState>(&conn)?;
    let qh = event_queue.handle();

    // Bind required protocols
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let seat_state = SeatState::new(&globals, &qh);
    let output_state = OutputState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);

    // Create a Wayland surface and layer surface on Layer::Top (always on top)
    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface.clone(), Layer::Top, Some("claude-poke"), None);
    layer.set_anchor(Anchor::TOP | Anchor::RIGHT);
    // Use logical size; compositor applies its own scale factor
    layer.set_size(logical_w, logical_h);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    // Set preferred buffer scale to 1 initially (updated when scale is detected)
    surface.set_buffer_scale(1);
    // Initial commit to trigger configure
    layer.commit();

    // Create softbuffer context and surface for software rendering (no GPU).
    // WaylandWindow is a thin wrapper around raw pointers — safe to create two
    // instances pointing to the same underlying wl_surface.
    let wayland_window_ctx = unsafe { WaylandWindow::new(&conn, &layer.wl_surface()) };
    let context = Context::new(wayland_window_ctx)
        .map_err(|e| anyhow::anyhow!("softbuffer Context::new: {e}"))?;

    // SAFETY: `context` is stored in LayerState and outlives the surface.
    // Both are dropped together when the thread exits.
    let ctx_ref: &'static Context<WaylandWindow> = unsafe { &*(&context as *const _) };
    let wayland_window_surface = unsafe { WaylandWindow::new(&conn, &layer.wl_surface()) };
    let soft_surface = Surface::new(ctx_ref, wayland_window_surface)
        .map_err(|e| anyhow::anyhow!("softbuffer Surface::new: {e}"))?;

    let buf_w = logical_w.max(1);
    let buf_h = logical_h.max(1);

    let mut state = LayerState {
        registry_state,
        seat_state,
        output_state,
        shm,

        layer,
        conn,
        logical_w,
        logical_h,
        scale: 1,
        width: logical_w,
        height: logical_h,
        configured: false,
        frame_data: None,
        exit: false,
        context,
        surface: Some(soft_surface),
        buffer: vec![0u32; (buf_w * buf_h) as usize],

        pointer: None,
    };

    info!("Layer surface created (logical {}x{}, softbuffer)", logical_w, logical_h);

    // Main event loop: dispatch Wayland events + render frames
    loop {
        // Non-blocking check for new frame data from main thread
        let mut new_frame = false;
        while let Ok(data) = frame_rx.try_recv() {
            match data {
                Some(frame) => {
                    state.frame_data = Some(frame);
                    new_frame = true;
                }
                None => {
                    info!("Render thread received exit signal");
                    return Ok(());
                }
            }
        }

        // Render only when we have new frame data and the surface is configured.
        // Avoids re-rendering the same frame 60 times/sec.
        if new_frame && state.configured {
            if let (Some(ref data), Some(ref mut surface)) = (&state.frame_data, &mut state.surface)
            {
                // Convert RGBA bytes to ARGB u32 and write to softbuffer buffer
                let w = state.width as usize;
                let h = state.height as usize;
                state.buffer.fill(0);
                for y in 0..h.min(state.height as usize) {
                    for x in 0..w.min(state.width as usize) {
                        let src_idx = (y * w + x) * 4;
                        let dst_idx = y * w + x;
                        if src_idx + 3 < data.len() && dst_idx < state.buffer.len() {
                            let r = data[src_idx] as u32;
                            let g = data[src_idx + 1] as u32;
                            let b = data[src_idx + 2] as u32;
                            let a = data[src_idx + 3] as u32;
                            state.buffer[dst_idx] = (a << 24) | (r << 16) | (g << 8) | b;
                        }
                    }
                }

                // Present the buffer
                if let Ok(mut sb_buf) = surface.buffer_mut() {
                    let len = state.buffer.len().min(sb_buf.len());
                    sb_buf[..len].copy_from_slice(&state.buffer[..len]);
                    if let Err(e) = sb_buf.present() {
                        warn!("Softbuffer present error: {}", e);
                    }
                }
                // Damage the entire surface
                state
                    .layer
                    .wl_surface()
                    .damage_buffer(0, 0, state.width as i32, state.height as i32);
                state.layer.commit();
            }
        }

        // Dispatch pending Wayland events (non-blocking)
        event_queue.dispatch_pending(&mut state)?;

        // Sleep until next poll — we only render on new frames, so a longer
        // sleep is fine. 50ms ≈ 20fps poll rate, adds at most 50ms display
        // latency which is imperceptible for a desktop pet.
        std::thread::sleep(std::time::Duration::from_millis(50));

        if state.exit {
            break;
        }
    }

    Ok(())
}

/// State for the layer surface event loop.
#[allow(dead_code)] // conn and context are kept alive for raw pointers / Surface borrow
struct LayerState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    layer: LayerSurface,
    conn: Connection,
    logical_w: u32,
    logical_h: u32,
    scale: i32,
    width: u32,
    height: u32,
    configured: bool,
    frame_data: Option<FrameData>,
    exit: bool,
    context: Context<WaylandWindow>,
    surface: Option<Surface<WaylandWindow, WaylandWindow>>,
    buffer: Vec<u32>,

    pointer: Option<wl_pointer::WlPointer>,
}

impl CompositorHandler for LayerState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if new_factor == self.scale {
            return;
        }
        info!("Compositor scale factor changed: {} -> {}", self.scale, new_factor);
        self.scale = new_factor;
        let new_w = self.logical_w * new_factor as u32;
        let new_h = self.logical_h * new_factor as u32;

        // Update buffer scale so compositor knows the buffer resolution
        surface.set_buffer_scale(new_factor);

        // Resize the softbuffer surface to the new physical size
        if let Some(ref mut soft_surface) = self.surface {
            if let (Some(nw), Some(nh)) = (NonZeroU32::new(new_w), NonZeroU32::new(new_h)) {
                if let Err(e) = soft_surface.resize(nw, nh) {
                    error!("Failed to resize softbuffer surface: {}", e);
                } else {
                    self.buffer.resize((new_w * new_h) as usize, 0);
                    self.width = new_w;
                    self.height = new_h;
                    // Resize the layer surface to the new physical size
                    self.layer.set_size(new_w, new_h);
                    self.layer.commit();
                    info!("Softbuffer surface resized to {}x{} (physical)", new_w, new_h);
                }
            }
        }
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for LayerState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for LayerState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 > 0 && configure.new_size.1 > 0 {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        self.configured = true;
        debug!("Layer surface configured: {}x{}", self.width, self.height);
    }
}

impl SeatHandler for LayerState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self.seat_state.get_pointer(qh, &seat).unwrap();
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            if let Some(pointer) = self.pointer.take() {
                pointer.release();
            }
        }
    }
}

impl PointerHandler for LayerState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                PointerEventKind::Press { button, .. } => {
                    debug!("Pointer press on layer surface: button={}", button);
                }
                PointerEventKind::Motion { .. } => {}
                _ => {}
            }
        }
    }
}

impl ShmHandler for LayerState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_shm!(LayerState);
delegate_seat!(LayerState);
delegate_pointer!(LayerState);
delegate_layer!(LayerState);
delegate_registry!(LayerState);

impl ProvidesRegistryState for LayerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
