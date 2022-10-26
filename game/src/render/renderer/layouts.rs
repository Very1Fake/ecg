use wgpu::Device;

use crate::render::pipelines::GlobalLayout;

pub struct Layouts {
    pub globals: GlobalLayout,
}

impl Layouts {
    pub fn new(device: &Device) -> Self {
        Self {
            globals: GlobalLayout::new(device),
        }
    }
}
