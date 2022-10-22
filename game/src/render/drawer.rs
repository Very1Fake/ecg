use wgpu::{IndexFormat, RenderPass};

use crate::scene::camera::CameraBind;

use super::model::{DrawModel, Model};

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
