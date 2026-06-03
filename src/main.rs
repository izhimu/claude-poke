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

    info!("Starting Claude Poke v{}", env!("CARGO_PKG_VERSION"));

    app::run()
}
