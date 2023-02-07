use thiserror::Error;
use wgpu::{RequestDeviceError, SurfaceError, CreateSurfaceError};

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
    #[error("Surface creation error: {0}")]
    CreateSurfaceError(CreateSurfaceError),
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

impl From<CreateSurfaceError> for RenderError {
    fn from(err: CreateSurfaceError) -> Self {
        Self::CreateSurfaceError(err)
    }
}
