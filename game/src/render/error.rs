use thiserror::Error;
use wgpu::{RequestDeviceError, SurfaceError};

/// Represents one of renderer errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to request a device: {0}")]
    RequestDeviceError(RequestDeviceError),
    #[error("Supported adapters not found")]
    AdapterNotFound,
    #[error("Compatible surface format not found")]
    NoCompatibleSurfaceFormat,
    #[error("Surface error: {0}")]
    SurfaceError(SurfaceError),
}

impl From<RequestDeviceError> for RenderError {
    fn from(err: RequestDeviceError) -> Self {
        Self::RequestDeviceError(err)
    }
}

impl From<SurfaceError> for RenderError {
    fn from(err: SurfaceError) -> Self {
        Self::SurfaceError(err)
    }
}
