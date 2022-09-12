use std::env::var;

use thiserror::Error;
use tracing::Level;
use tracing_subscriber::fmt::fmt;

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("Can't parse log level (found: {0:?})")]
    LogLevelError(Option<String>),
}

pub fn bootstrap() -> Result<(), BootstrapError> {
    fmt()
        .with_max_level({
            match var("LOG_LEVEL") {
                Ok(level) => match level.to_lowercase().as_str() {
                    "trace" => Level::TRACE,
                    "debug" => Level::DEBUG,
                    "info" => Level::INFO,
                    "warn" => Level::WARN,
                    "error" => Level::ERROR,
                    _ => return Err(BootstrapError::LogLevelError(Some(level))),
                },
                #[cfg(debug_assertions)]
                Err(_) => Level::TRACE,
                #[cfg(not(debug_assertions))]
                Err(_) => Level::INFO,
            }
        })
        .init();

    Ok(())
}
