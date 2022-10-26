#![windows_subsystem = "windows"]

pub mod bootstrap;
#[cfg(feature = "debug_overlay")]
pub mod egui;
pub mod game;
pub mod render;
pub mod run;
pub mod scene;
pub mod types;
pub mod utils;
pub mod window;

use anyhow::{Context, Result};
use tokio::runtime::Builder;
use tracing::{debug, info};

use bootstrap::bootstrap;
use run::run;

use crate::{game::Game, render::renderer::Renderer, utils::VERSION, window::Window};

// TODO: Drop anyhow
fn main() -> Result<()> {
    bootstrap()?;

    info!("Starting game instance. ECG v{VERSION}");

    let runtime = Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .build()?;
    let (window, event_loop) = Window::new().with_context(|| "While creating game window")?;

    let renderer = Renderer::new(&window.inner, &runtime)?;

    let game = Game::new(&window, renderer);

    debug!("Game starts");
    runtime.block_on(run(event_loop, window, game))?;

    Ok(())
}
