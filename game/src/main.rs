#![windows_subsystem = "windows"]

use tokio::runtime::Builder;
use tracing::{debug, info};

use ecg_game::{bootstrap::bootstrap, error::Error, utils::VERSION, window::Window, Game};

// TODO: Drop anyhow
fn main() -> Result<(), Error> {
    bootstrap()?;

    #[cfg(feature = "tracy")]
    {
        debug!("Starting profiling client");
        let _client = tracy_client::Client::start();
    }

    info!("Starting game instance. ECG v{VERSION}");

    let runtime = Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .build()
        .unwrap();
    let (window, event_loop) = Window::new(&runtime)?;

    let game = Game::new(window, runtime);

    debug!("Game starts");
    game.run(event_loop);

    Ok(())
}
