use crate::{bootstrap::BootstrapError, render::error::RenderError};

#[derive(Debug)]
pub enum Error {
    /// Error related to bootstrapping
    BootstrapError(BootstrapError),
    /// Error re
    RenderError(RenderError),
}

impl From<BootstrapError> for Error {
    fn from(err: BootstrapError) -> Self {
        Self::BootstrapError(err)
    }
}

impl From<RenderError> for Error {
    fn from(err: RenderError) -> Self {
        Self::RenderError(err)
    }
}
