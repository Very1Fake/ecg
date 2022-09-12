pub mod bootstrap;
pub mod game;
pub mod meta;
pub mod run;
pub mod window;

use anyhow::{Result, Context};
use tracing::info;

use bootstrap::bootstrap;
use meta::VERSION;
use run::run;

use crate::window::Window;

fn main() -> Result<()> {
    bootstrap()?;

    info!("Starting game instance. ECG v{VERSION}");

    run(Window::new().with_context(|| "While creating game window")?);

    Ok(())
}
