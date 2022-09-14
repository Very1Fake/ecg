pub mod bootstrap;
pub mod game;
pub mod graphics;
pub mod run;
pub mod window;
pub mod utils;

use anyhow::{Context, Result};
use tokio::runtime::Builder;
use tracing::{info, debug};

use bootstrap::bootstrap;
use run::run;

use crate::{game::Game, graphics::Graphics, window::Window, utils::VERSION};

fn main() -> Result<()> {
    bootstrap()?;

    info!("Starting game instance. ECG v{VERSION}");

    let runtime = Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .build()?;
    let window = Window::new().with_context(|| "While creating game window")?;

    debug!("Connecting to GPU");
    let graphics = runtime.block_on(Graphics::new(&window.inner))?;

    debug!("Game starts");
    runtime.block_on(run(window, Game::new(graphics)));

    Ok(())
}
