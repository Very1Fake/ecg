use std::ops::Range;

use wgpu::{Buffer, IndexFormat, RenderPass};

use crate::scene::camera::CameraBind;

// TODO: Static model mega-buffer
pub trait Model {
    const INDEX_FORMAT: IndexFormat = IndexFormat::Uint16;

    fn get_vertices(&self) -> &Buffer;

    fn get_indices(&self) -> (&Buffer, u32);
}

pub trait DrawModel<'a> {
    fn draw_model<T: Model>(&mut self, model: &'a T, camera_bind: &'a CameraBind);

    fn draw_model_instanced<T: Model>(
        &mut self,
        model: &'a T,
        instances: Range<u32>,
        camera_bind: &'a CameraBind,
    );
}

impl<'a, 'b> DrawModel<'b> for RenderPass<'a>
where
    'b: 'a,
{
    fn draw_model<T: Model>(&mut self, model: &'b T, camera_bind: &'b CameraBind) {
        self.draw_model_instanced(model, 0..1, camera_bind)
    }

    fn draw_model_instanced<T: Model>(
        &mut self,
        model: &'a T,
        instances: std::ops::Range<u32>,
        camera_bind: &'a CameraBind,
    ) {
        let (index_buffer, count) = model.get_indices();

        self.set_vertex_buffer(0, model.get_vertices().slice(..));
        self.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
        self.set_bind_group(0, &camera_bind.bind_group, &[]);
        self.draw_indexed(0..count, 0, instances);
    }
}
