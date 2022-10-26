use crate::render::pipelines::{GlobalModel, GlobalsBindGroup};

use super::Renderer;

impl Renderer {
    pub fn bind_globals(&self, global_model: &GlobalModel) -> GlobalsBindGroup {
        self.layouts
            .globals
            .bind_globals(&self.device, global_model)
    }
}
