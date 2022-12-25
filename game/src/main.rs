#![windows_subsystem = "windows"]

use tokio::runtime::Builder;
use tracing::{debug, info};

use ecg_game::{
    bootstrap::bootstrap,
    consts::{ASYNC_THREADS, BLOCKING_THREADS},
    error::Error,
    utils::VERSION,
    window::Window,
    Game,
};

#[cfg_attr(feature = "tracy-memory", global_allocator)]
#[cfg(feature = "tracy-memory")]
static GLOBAL: common_log::tracy_client::ProfiledAllocator<std::alloc::System> =
    common_log::tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

fn main() -> Result<(), Error> {
    bootstrap()?;

    #[cfg(feature = "tracy")]
    {
        debug!("Starting profiling client");
        let _client = tracy_client::Client::start();
    }

    info!("Starting game instance. ECG v{VERSION}");

    let runtime = Builder::new_multi_thread()
        .worker_threads(ASYNC_THREADS)
        .max_blocking_threads(*BLOCKING_THREADS)
        .build()
        .unwrap();
    let (window, event_loop) = Window::new(&runtime)?;

    let game = Game::new(window, runtime);

    debug!("Game starts");
    game.run(event_loop);

    Ok(())
}
