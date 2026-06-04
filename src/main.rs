mod app;
mod animation;
mod config;
mod monitor;
mod render;
mod state;
mod system;

use log::info;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Initialize GTK before winit takes over the event loop (required for tray on Linux).
    // libappindicator uses D-Bus (StatusNotifierItem) to communicate with the desktop
    // shell, and it needs the GLib main context pumped to process those messages.
    // We iterate the context from the winit event loop in App::about_to_wait().
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = gtk::init() {
            log::warn!("GTK init failed (tray may not work): {}", e);
        }
    }

    info!("Starting Claude Poke v{}", env!("CARGO_PKG_VERSION"));

    app::run()
}
