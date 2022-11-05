#![windows_subsystem = "windows"]

pub mod bootstrap;
pub mod consts;
#[cfg(feature = "debug_overlay")]
pub mod egui;
pub mod game;
pub mod render;
pub mod scene;
pub mod types;
pub mod utils;
pub mod window;

use anyhow::{Context, Result};
use tokio::runtime::Builder;
use tracing::{debug, info};

use bootstrap::bootstrap;

use crate::{game::Game, utils::VERSION, window::Window};

// TODO: Drop anyhow
fn main() -> Result<()> {
    bootstrap()?;

    info!("Starting game instance. ECG v{VERSION}");

    let runtime = Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .build()?;
    let (window, event_loop) =
        Window::new(&runtime).with_context(|| "While creating game window")?;

    let game = Game::new(window, runtime);

    debug!("Game starts");
    game.run(event_loop)?;

    Ok(())
}
