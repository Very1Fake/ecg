use thiserror::Error;
use wgpu::RequestDeviceError;

/// Represents one of renderer errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to request a device")]
    RequestDeviceError(RequestDeviceError),
    #[error("Supported adapters not found")]
    AdapterNotFound,
    #[error("Compatible surface format not found")]
    NoCompatibleSurfaceFormat,
}

impl From<RequestDeviceError> for RenderError {
    fn from(err: RequestDeviceError) -> Self {
        Self::RequestDeviceError(err)
    }
}
