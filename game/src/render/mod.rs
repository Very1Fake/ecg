use wgpu::PresentMode;

pub mod buffer;
pub mod error;
pub mod model;
pub mod pipelines;
pub mod primitives;
pub mod renderer;
pub mod shader;
pub mod texture;

#[derive(PartialEq, Eq, Clone)]
pub struct RenderMode {
    pub present_mode: PresentMode,
}

impl RenderMode {
    pub const fn new() -> Self {
        Self {
            present_mode: PresentMode::Fifo,
        }
    }
}
