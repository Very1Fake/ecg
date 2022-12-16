use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use common::{
    block::Block,
    coord::{BlockCoord, ChunkId, GlobalCoord, GlobalUnit, CHUNK_CUBE},
    span,
};
use tokio::runtime::Runtime;
use wgpu::{BufferUsages, Device};

use crate::{
    render::{
        buffer::Buffer,
        mesh::{MeshTaskResult, TerrainMesh},
        primitives::vertex::Vertex,
    },
    types::F32x3,
};

use super::camera::Camera;

pub struct ChunkManager {
    // TODO: Move to game settings
    pub draw_distance: u16,

    pub mesh_builder_rx: Receiver<MeshTaskResult>,
    pub mesh_builder_tx: Sender<MeshTaskResult>,

    pub logic: HashMap<ChunkId, LogicChunk>,
    pub terrain: HashMap<ChunkId, TerrainChunk>,
}

impl ChunkManager {
    // Limits
    pub const MAX_LOADS_PER_FRAME: usize = 2;
    pub const MAX_UNLOADS_PER_FRAME: usize = 4;
    pub const MIN_DRAW_DISTANCE: u16 = 2;
    pub const MAX_DRAW_DISTANCE: u16 = 256;

    pub fn new() -> Self {
        let (mesh_builder_tx, mesh_builder_rx) = channel();

        Self {
            draw_distance: Self::MIN_DRAW_DISTANCE,

            mesh_builder_rx,
            mesh_builder_tx,

            logic: HashMap::new(),
            terrain: HashMap::new(),
        }
    }

    /// Maintain chunk manager. Regenerate chunk meshes.
    pub fn maintain(&mut self, device: &Device, runtime: &Runtime, camera: &Camera) {
        span!(_guard, "maintain", "ChunkManager::maintain");

        // Collect generated meshes
        self.mesh_builder_rx
            .try_iter()
            .take(4)
            .for_each(|(coord, mesh)| {
                // TODO: Check if terrain already rebuilt
                self.terrain
                    .insert(coord.to_id(), TerrainChunk::new(device, mesh));
            });

        // Run mesh generating tasks
        self.logic
            .iter_mut()
            .filter(|(_, chunk)| chunk.is_dirty())
            .for_each(|(coord, chunk)| {
                // TODO: Add a check for an empty mesh when it'll be aware of neighboring blocks
                // Check if chunk has at least one opaque block. Otherwise skip mesh building
                if chunk.blocks.iter().any(|block| block.opaque()) {
                    let tx = self.mesh_builder_tx.clone();
                    let coord = *coord;
                    let blocks = chunk.blocks;
                    runtime.spawn_blocking(move || {
                        TerrainMesh::task(tx, coord.to_coord(), &blocks);
                    });
                }
                chunk.dirty = false;
            });

        self.load_chunks(camera.pos);
    }

    pub fn load_chunks(&mut self, player_pos: F32x3) {
        LoadArea::new(
            // FIX
            GlobalCoord::from_vec3(player_pos).to_chunk_id(),
            self.draw_distance as i64,
        )
        .filter(|coord| !self.logic.contains_key(coord))
        .take(Self::MAX_LOADS_PER_FRAME)
        .collect::<Vec<_>>()
        .iter()
        .for_each(|coord| {
            self.logic.insert(*coord, generate_chunk(*coord));
        });
    }

    pub fn cleanup(&mut self) {
        // BUG: No freeing
        self.logic.shrink_to_fit();
        self.terrain.shrink_to_fit();
    }

    pub fn clear_mesh(&mut self) {
        self.logic.values_mut().for_each(|chunk| chunk.dirty = true);
        self.terrain.clear();
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents chunk state
pub struct LogicChunk {
    blocks: [Block; CHUNK_CUBE],
    dirty: bool,
}

impl LogicChunk {
    pub fn new() -> Self {
        Self {
            blocks: [Block::Air; CHUNK_CUBE],
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn blocks_mut(&mut self) -> &mut [Block; CHUNK_CUBE] {
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

pub struct LoadArea {
    start: ChunkId,
    end: ChunkId,
    current: ChunkId,
}

impl LoadArea {
    pub fn new(center: ChunkId, radius: GlobalUnit) -> Self {
        let start = ChunkId::new(center.x - radius, center.y - radius, center.z - radius);
        let end = ChunkId::new(center.x + radius, center.y + radius, center.z + radius);

        Self {
            start,
            end,
            current: start,
        }
    }
}

impl Iterator for LoadArea {
    type Item = ChunkId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.z > self.end.z {
            return None;
        }

        let item = self.current;
        let mut new = self.current;

        fn clamped_inc(src: &mut GlobalUnit, clamp: GlobalUnit) -> bool {
            if *src < clamp {
                *src += 1;
                false
            } else {
                true
            }
        }

        if clamped_inc(&mut new.x, self.end.x) {
            new.x = self.start.x;
            if clamped_inc(&mut new.y, self.end.y) {
                new.y = self.start.y;
                clamped_inc(&mut new.z, self.end.z + 1);
            }
        }

        self.current = new;
        Some(item)
    }
}

fn generate_chunk(c_id: ChunkId) -> LogicChunk {
    let mut chunk = LogicChunk::new();
    let coord = c_id.to_coord();

    chunk.blocks.iter_mut().enumerate().for_each(|(i, block)| {
        let pos = coord.to_global(&BlockCoord::from(i));

        *block = match pos.y {
            0 => Block::Grass,
            -10..=-1 => Block::Dirt,
            -128..=-11 => Block::Stone,
            GlobalUnit::MIN..=-129 => Block::Stone,
            _ => Block::Air,
        };
    });

    chunk
}

#[cfg(test)]
mod tests {
    use common::coord::ChunkId;

    use super::LoadArea;

    #[test]
    fn load_area() {
        let chunks = LoadArea::new(ChunkId::ZERO, 1).collect::<Vec<_>>();

        eprintln!("{}", chunks.len());

        assert_eq!(
            chunks,
            [
                ChunkId::new(-1, -1, -1),
                ChunkId::new(0, -1, -1),
                ChunkId::new(1, -1, -1),
                ChunkId::new(-1, 0, -1),
                ChunkId::new(0, 0, -1),
                ChunkId::new(1, 0, -1),
                ChunkId::new(-1, 1, -1),
                ChunkId::new(0, 1, -1),
                ChunkId::new(1, 1, -1),
                ChunkId::new(-1, -1, 0),
                ChunkId::new(0, -1, 0),
                ChunkId::new(1, -1, 0),
                ChunkId::new(-1, 0, 0),
                ChunkId::ZERO,
                ChunkId::new(1, 0, 0),
                ChunkId::new(-1, 1, 0),
                ChunkId::new(0, 1, 0),
                ChunkId::new(1, 1, 0),
                ChunkId::new(-1, -1, 1),
                ChunkId::new(0, -1, 1),
                ChunkId::new(1, -1, 1),
                ChunkId::new(-1, 0, 1),
                ChunkId::new(0, 0, 1),
                ChunkId::new(1, 0, 1),
                ChunkId::new(-1, 1, 1),
                ChunkId::new(0, 1, 1),
                ChunkId::new(1, 1, 1),
            ]
        );
    }
}
