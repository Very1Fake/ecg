use wgpu::{Buffer, IndexFormat};

// TODO: Static model mega-buffer
pub trait Model {
    const INDEX_FORMAT: IndexFormat = IndexFormat::Uint16;

    fn get_vertices(&self) -> &Buffer;

    fn get_indices(&self) -> (&Buffer, u32);
}
