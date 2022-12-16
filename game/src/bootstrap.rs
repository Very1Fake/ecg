use std::{env::var, str::FromStr};

use thiserror::Error;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt::fmt, EnvFilter};

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("Can't parse log level (found: {0:?})")]
    LogLevelError(Option<String>),
}

pub const DEFAULT_LOG_FILTER: &[&str] = &[
    "wgpu_core=info",
    "wgpu_core::instance=warn",
    "wgpu_core::device=warn",
    "wgpu_hal=info",
    "naga=info",
];

pub fn bootstrap() -> Result<(), BootstrapError> {
    let mut filter = EnvFilter::default().add_directive(
        match var("LOG_LEVEL") {
            Ok(level) => match LevelFilter::from_str(level.to_lowercase().as_str()) {
                Ok(level) => level,
                Err(_) => return Err(BootstrapError::LogLevelError(Some(level))),
            },
            #[cfg(debug_assertions)]
            Err(_) => LevelFilter::TRACE,
            #[cfg(not(debug_assertions))]
            Err(_) => LevelFilter::INFO,
        }
        .into(),
    );

    for dir in DEFAULT_LOG_FILTER {
        filter = filter.add_directive(dir.parse().unwrap());
    }

    // TODO: Add log file support
    fmt().with_env_filter(filter).init();

    Ok(())
}
