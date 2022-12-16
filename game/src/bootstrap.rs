use std::env::var;

use thiserror::Error;
use tracing_subscriber::fmt::fmt;

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("Can't parse log level (found: {0:?})")]
    LogLevelError(Option<String>),
}

pub fn bootstrap() -> Result<(), BootstrapError> {
    fmt()
        .with_env_filter(format!(
            "{},wgpu_core=info,wgpu_hal=info,naga=info",
            match var("LOG_LEVEL") {
                Ok(level) => {
                    match level.to_lowercase().as_str() {
                        "trace" | "debug" | "info" | "warn" | "error" => level,
                        _ => return Err(BootstrapError::LogLevelError(Some(level))),
                    }
                }
                #[cfg(debug_assertions)]
                Err(_) => String::from("trace"),
                #[cfg(not(debug_assertions))]
                Err(_) => String::from("info"),
            }
        ))
        .init();

    Ok(())
}
