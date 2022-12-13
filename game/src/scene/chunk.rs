use std::collections::HashMap;

use common::{
    block::Block,
    coord::{ChunkId, CHUNK_CUBE},
    span,
};
use wgpu::{BufferUsages, Device};

use crate::render::{buffer::Buffer, mesh::TerrainMesh, primitives::vertex::Vertex};

#[derive(Default)]
pub struct ChunkManager {
    pub logic: HashMap<ChunkId, LogicChunk>,
    pub terrain: HashMap<ChunkId, TerrainChunk>,
}

impl ChunkManager {
    /// Maintain chunk manager. Regenerate chunk meshes.
    pub fn maintain(&mut self, device: &Device) {
        span!(_guard, "maintain", "ChunkManager::maintain");

        self.logic
            .iter_mut()
            .filter(|(_, chunk)| chunk.is_dirty())
            .for_each(|(coord, chunk)| {
                // TODO: Add a check for an empty mesh when it'll be aware of neighboring blocks
                // Check if chunk has at least one opaque block. Otherwise skip mesh building
                if chunk.blocks.iter().any(|block| block.opaque()) {
                    let mesh = TerrainMesh::build(coord.to_coord(), &chunk.blocks);
                    tracing::debug!(?coord, "Building mesh for chunk");

                    self.terrain.insert(*coord, TerrainChunk::new(device, mesh));
                } else {
                    self.terrain.remove(coord);
                }

                chunk.dirty = false;
            });
    }
}

/// Represents chunk state
pub struct LogicChunk {
    blocks: Box<[Block]>,
    dirty: bool,
}

impl LogicChunk {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([Block::Air; CHUNK_CUBE]),
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn blocks_mut(&mut self) -> &mut [Block] {
        self.dirty = true;
        &mut self.blocks
    }

    pub fn blocks_box(&mut self) -> &mut Box<[Block]> {
        self.dirty = true;
        &mut self.blocks
    }
}

impl Default for LogicChunk {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents chunk mesh on GPU
pub struct TerrainChunk {
    pub vertex_buffer: Buffer<Vertex>,
    pub index_buffer: Buffer<u32>,
}

impl TerrainChunk {
    pub fn new(device: &Device, mesh: TerrainMesh) -> Self {
        Self {
            vertex_buffer: Buffer::new(device, &mesh.vertices, BufferUsages::VERTEX),
            index_buffer: Buffer::new(device, &mesh.indices, BufferUsages::INDEX),
        }
    }
}
