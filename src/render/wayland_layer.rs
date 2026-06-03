use super::wayland_window::WaylandWindow;
use crate::config::WindowConfig;
use anyhow::Result;
use log::{debug, error, info, warn};
use pixels::{PixelsBuilder, SurfaceTexture};
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
    let scale = config.scale;

    std::thread::Builder::new()
        .name("wayland-layer".into())
        .spawn(move || {
            if let Err(e) = run_layer_surface(width, height, scale, rx) {
                error!("Layer surface render thread error: {}", e);
            }
        })?;

    Ok(tx)
}

/// Internal: run the layer surface event loop on a dedicated thread.
fn run_layer_surface(
    logical_w: u32,
    logical_h: u32,
    scale: u32,
    frame_rx: mpsc::Receiver<Option<FrameData>>,
) -> Result<()> {
    let physical_w = logical_w * scale;
    let physical_h = logical_h * scale;

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
    layer.set_size(physical_w, physical_h);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    // Initial commit to trigger configure
    layer.commit();

    // Create a pixels renderer using the layer surface's wl_surface
    let wayland_window = unsafe { WaylandWindow::new(&conn, &layer.wl_surface()) };
    let surface_texture = SurfaceTexture::new(physical_w, physical_h, &wayland_window);
    let mut pixels = PixelsBuilder::new(logical_w, logical_h, surface_texture)
        .clear_color(pixels::wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        })
        .alpha_mode(pixels::wgpu::CompositeAlphaMode::PreMultiplied)
        .build()?;

    let mut state = LayerState {
        registry_state,
        seat_state,
        output_state,
        shm,

        layer,
        width: physical_w,
        height: physical_h,
        configured: false,
        frame_data: None,
        exit: false,

        pointer: None,
    };

    info!(
        "Layer surface created ({}x{}, logical {}x{})",
        physical_w, physical_h, logical_w, logical_h
    );

    // Main event loop: dispatch Wayland events + render frames
    loop {
        // Non-blocking check for new frame data from main thread
        while let Ok(data) = frame_rx.try_recv() {
            match data {
                Some(frame) => state.frame_data = Some(frame),
                None => {
                    info!("Render thread received exit signal");
                    return Ok(());
                }
            }
        }

        // Render if we have frame data and the surface is configured
        if state.configured {
            if let Some(ref data) = state.frame_data {
                let frame = pixels.frame_mut();
                let len = frame.len().min(data.len());
                frame[..len].copy_from_slice(&data[..len]);
                if let Err(e) = pixels.render() {
                    warn!("Render error: {}", e);
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

        // Small sleep to avoid busy-spinning (~60fps)
        std::thread::sleep(std::time::Duration::from_millis(16));

        if state.exit {
            break;
        }
    }

    Ok(())
}

/// State for the layer surface event loop.
struct LayerState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    layer: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    frame_data: Option<FrameData>,
    exit: bool,

    pointer: Option<wl_pointer::WlPointer>,
}

impl CompositorHandler for LayerState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
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
