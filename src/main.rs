pub mod bootstrap;
pub mod game;
pub mod graphics;
pub mod run;
pub mod utils;
pub mod window;

use anyhow::{Context, Result};
use tokio::runtime::Builder;
use tracing::{debug, info};

use bootstrap::bootstrap;
use run::run;

use crate::{game::Game, graphics::Graphics, utils::VERSION, window::Window};

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
