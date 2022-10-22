use std::ops::Range;

use wgpu::{Buffer, IndexFormat};

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
