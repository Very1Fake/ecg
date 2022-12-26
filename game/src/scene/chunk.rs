use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{channel, Receiver, Sender},
};

use common::{
    block::Block,
    coord::{BlockCoord, ChunkId, GlobalCoord, GlobalUnit, CHUNK_CUBE, CHUNK_SIZE, CHUNK_SQUARE},
    direction::Direction,
};
use common_log::{prof, span};
use tokio::runtime::Runtime;
use wgpu::{BufferUsages, Device};

use crate::{
    consts::{BLOCKING_THREADS, CPU_CORES},
    render::{
        buffer::Buffer,
        mesh::{MeshTaskResult, Neighbors, TerrainMesh},
        primitives::vertex::Vertex,
    },
};

use super::camera::Camera;

pub struct ChunkManager {
    // TODO: Move to game settings
    pub draw_distance: u16,

    pub mesh_builder_rx: Receiver<MeshTaskResult>,
    pub mesh_builder_tx: Sender<MeshTaskResult>,

    pub chunk_gen_rx: Receiver<(ChunkId, LogicChunk)>,
    pub chunk_gen_tx: Sender<(ChunkId, LogicChunk)>,
    pub chunk_gen_ids: HashSet<ChunkId>,

    pub logic: HashMap<ChunkId, LogicChunk>,
    pub terrain: HashMap<ChunkId, TerrainChunk>,
}

impl ChunkManager {
    // Limits
    pub const MIN_DRAW_DISTANCE: u16 = 2;
    pub const MAX_DRAW_DISTANCE: u16 = 256;

    pub fn new() -> Self {
        let (mesh_builder_tx, mesh_builder_rx) = channel();
        let (chunk_gen_tx, chunk_gen_rx) = channel();

        Self {
            draw_distance: Self::MIN_DRAW_DISTANCE,

            mesh_builder_rx,
            mesh_builder_tx,

            chunk_gen_rx,
            chunk_gen_tx,
            chunk_gen_ids: HashSet::with_capacity(*BLOCKING_THREADS * 4),

            logic: HashMap::new(),
            terrain: HashMap::new(),
        }
    }

    /// Maintain chunk manager. Regenerate chunk meshes.
    pub fn maintain(&mut self, device: &Device, runtime: &Runtime, camera: &Camera) {
        span!(_guard, "maintain", "ChunkManager::maintain");

        // Collect generated terrain chunks
        self.mesh_builder_rx.try_iter().for_each(|(coord, mesh)| {
            let coord = coord.to_id();

            // TODO: Check if terrain already rebuilt
            if let Some(logic) = self.logic.get_mut(&coord) {
                if matches!(logic.status, TerrainStatus::Pending) {
                    self.terrain.insert(coord, TerrainChunk::new(device, mesh));
                    logic.status = TerrainStatus::Built;
                } else {
                    tracing::warn!(?coord, "Chunk mesh building collision");
                }
            }
        });

        // Collect generated logic chunks
        self.chunk_gen_rx.try_iter().for_each(|(id, chunk)| {
            self.chunk_gen_ids.remove(&id);
            self.logic.insert(id, chunk);
        });

        // Run mesh generating tasks
        self.logic
            .iter_mut()
            .filter(|(_, chunk)| matches!(chunk.status, TerrainStatus::None))
            .take(*BLOCKING_THREADS * 8)
            .for_each(|(coord, chunk)| {
                // TODO: Add a check for an empty mesh when it'll be aware of neighboring blocks
                // Check if chunk has at least one opaque block. Otherwise skip mesh building
                if chunk.blocks.iter().any(|block| block.opaque()) {
                    let tx = self.mesh_builder_tx.clone();
                    let coord = *coord;
                    let blocks = chunk.blocks;
                    runtime.spawn_blocking(move || {
                        TerrainMesh::task(tx, coord.to_coord(), &blocks, Neighbors::default());
                    });

                    chunk.status = TerrainStatus::Pending;
                } else {
                    // Free old mesh buffer for updated empty chunk
                    self.terrain.remove(coord);
                    chunk.status = TerrainStatus::Built;
                }
            });

        // Load new chunks
        LoadArea::new_cuboid(
            GlobalCoord::from_vec3(camera.pos).to_chunk_id(),
            self.draw_distance as i64,
        )
        .collect::<Vec<_>>()
        .iter()
        .filter(|id| {
            !self.logic.contains_key(id)
                && !self.chunk_gen_ids.contains(id)
                && self.chunk_gen_ids.len() < *CPU_CORES
        })
        .take(*BLOCKING_THREADS * 4 - self.chunk_gen_ids.len())
        .collect::<Vec<_>>()
        .iter()
        .for_each(|&&id| {
            self.chunk_gen_ids.insert(id);

            let tx = self.chunk_gen_tx.clone();
            runtime.spawn_blocking(move || {
                let _ = tx.send((id, LogicChunk::generate_flat(id)));
            });
        });

        // Unload old chunks
        let load_area = LoadArea::new_cuboid(
            GlobalCoord::from_vec3(camera.pos).to_chunk_id(),
            self.draw_distance as i64,
        );
        self.logic
            .keys()
            .filter(|&id| !load_area.contains(*id))
            .copied()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|id| {
                self.logic.remove(id);
                self.terrain.remove(id);
            });
    }

    pub fn cleanup(&mut self) {
        self.logic.shrink_to_fit();
        self.terrain.shrink_to_fit();
    }

    pub fn clear_mesh(&mut self) {
        self.logic
            .values_mut()
            .for_each(|chunk| chunk.status = TerrainStatus::None);
        self.terrain.clear();
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default)]
pub enum TerrainStatus {
    #[default]
    None,
    Pending,
    Built,
}

/// Represents chunk state
pub struct LogicChunk {
    blocks: [Block; CHUNK_CUBE],
    status: TerrainStatus,
}

impl LogicChunk {
    pub const fn new() -> Self {
        Self {
            blocks: [Block::Air; CHUNK_CUBE],
            status: TerrainStatus::None,
        }
    }

    pub const fn from_blocks(blocks: [Block; CHUNK_CUBE]) -> Self {
        Self {
            blocks,
            status: TerrainStatus::None,
        }
    }

    pub fn status(&self) -> TerrainStatus {
        self.status
    }

    pub fn blocks_mut(&mut self) -> &mut [Block; CHUNK_CUBE] {
        self.status = TerrainStatus::None;
        &mut self.blocks
    }

    pub fn edge(&self, dir: Direction) -> Vec<Block> {
        match dir {
            Direction::Down => self
                .blocks
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(i, b)| {
                    if i % CHUNK_SQUARE < CHUNK_SIZE {
                        Some(b)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Direction::Up => self
                .blocks
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(i, b)| {
                    if (i % CHUNK_SQUARE) / (CHUNK_SQUARE - CHUNK_SIZE) == 1 {
                        Some(b)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Direction::Left => self
                .blocks
                .iter()
                .copied()
                .skip(CHUNK_SIZE - 1)
                .step_by(CHUNK_SIZE)
                .collect::<Vec<_>>(),
            Direction::Right => self
                .blocks
                .iter()
                .copied()
                .step_by(CHUNK_SIZE)
                .collect::<Vec<_>>(),
            Direction::Front => self.blocks[..CHUNK_SQUARE].to_vec(),
            Direction::Back => self.blocks[(CHUNK_CUBE - CHUNK_SQUARE)..].to_vec(),
        }
    }

    fn generate_flat(id: ChunkId) -> LogicChunk {
        prof!("LogicChunk::generate_flat");

        let coord = id.to_coord();
        let mut blocks = [Block::Air; CHUNK_CUBE];

        blocks.iter_mut().enumerate().for_each(|(i, block)| {
            let pos = coord.to_global(&BlockCoord::from(i));

            match pos.y {
                0 => *block = Block::Grass,
                -10..=-1 => *block = Block::Dirt,
                -128..=-11 => *block = Block::Stone,
                GlobalUnit::MIN..=-129 => *block = Block::Stone,
                _ => {}
            };
        });

        LogicChunk::from_blocks(blocks)
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

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LoadArea {
    start: ChunkId,
    end: ChunkId,
    current: ChunkId,
}

impl LoadArea {
    const fn new(start: ChunkId, end: ChunkId) -> Self {
        Self {
            start,
            end,
            current: start,
        }
    }

    pub fn new_cube(center: ChunkId, dist: GlobalUnit) -> Self {
        Self::new(
            ChunkId::new(center.x - dist, center.y - dist, center.z - dist),
            ChunkId::new(center.x + dist, center.y + dist, center.z + dist),
        )
    }

    pub fn new_cuboid(center: ChunkId, dist: GlobalUnit) -> Self {
        Self::new(
            ChunkId::new(center.x - dist, center.y - dist / 2, center.z - dist),
            ChunkId::new(center.x + dist, center.y + dist / 2, center.z + dist),
        )
    }

    pub fn contains(&self, id: ChunkId) -> bool {
        !(id.x < self.start.x
            || id.x > self.end.x
            || id.y < self.start.y
            || id.y > self.end.y
            || id.z < self.start.z
            || id.z > self.end.z)
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

#[cfg(test)]
mod tests {
    use common::{block::Block, coord::ChunkId};

    use super::LoadArea;

    #[test]
    fn load_area_iter_cube() {
        let loaded_area = LoadArea::new_cube(ChunkId::ZERO, 1).collect::<Vec<_>>();

        assert_eq!(
            loaded_area,
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

    #[test]
    fn load_area_iter_cuboid() {
        let loaded_area = LoadArea::new_cuboid(ChunkId::ZERO, 1).collect::<Vec<_>>();

        assert_eq!(
            loaded_area,
            [
                ChunkId::new(-1, 0, -1),
                ChunkId::new(0, 0, -1),
                ChunkId::new(1, 0, -1),
                ChunkId::new(-1, 0, 0),
                ChunkId::ZERO,
                ChunkId::new(1, 0, 0),
                ChunkId::new(-1, 0, 1),
                ChunkId::new(0, 0, 1),
                ChunkId::new(1, 0, 1),
            ]
        );
    }

    #[test]
    fn load_area_contains() {
        let load_area = LoadArea::new_cube(ChunkId::ZERO, 2);

        assert!(load_area.contains(ChunkId::ZERO));
        assert!(load_area.contains(ChunkId::new(1, 1, 1)));
        assert!(!load_area.contains(ChunkId::new(3, 3, 3)));
        assert!(!load_area.contains(ChunkId::new(3, 32, 12)));
    }

    const SIZE: usize = 3;
    const SIZE_SQUARE: usize = SIZE.pow(2);
    const SIZE_CUBE: usize = SIZE.pow(3);
    #[rustfmt::skip]
    const BLOCKS: [Block; SIZE_CUBE] = [
        Block::Air, Block::Stone, Block::Air,
        Block::Stone, Block::Air, Block::Stone,
        Block::Air, Block::Stone, Block::Sand,

        Block::Stone, Block::Leaves, Block::Stone,
        Block::Grass, Block::Stone, Block::Dirt,
        Block::Stone, Block::Air, Block::Stone,

        Block::Air, Block::Stone, Block::Air,
        Block::Stone, Block::Air, Block::Stone,
        Block::Air, Block::Stone, Block::SandStone,
    ];

    #[test]
    fn neighbor_up() {
        assert_eq!(
            BLOCKS
                .into_iter()
                .enumerate()
                .filter_map(|(i, b)| if (i % SIZE_SQUARE) / (SIZE_SQUARE - SIZE) == 1 {
                    Some(b)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            [
                Block::Air,
                Block::Stone,
                Block::Sand,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::SandStone,
            ]
        );
    }

    #[test]
    fn neighbor_down() {
        assert_eq!(
            BLOCKS
                .into_iter()
                .enumerate()
                .filter_map(|(i, b)| if i % SIZE_SQUARE < SIZE {
                    Some(b)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            [
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Leaves,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
            ]
        );
    }

    #[test]
    fn neighbor_left() {
        assert_eq!(
            BLOCKS
                .into_iter()
                .skip(SIZE - 1)
                .step_by(SIZE)
                .collect::<Vec<_>>(),
            [
                Block::Air,
                Block::Stone,
                Block::Sand,
                Block::Stone,
                Block::Dirt,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::SandStone,
            ]
        );
    }

    #[test]
    fn neighbor_right() {
        assert_eq!(
            BLOCKS.into_iter().step_by(SIZE).collect::<Vec<_>>(),
            [
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Grass,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
            ]
        );
    }

    #[test]
    fn neighbor_front() {
        assert_eq!(
            BLOCKS[..SIZE_SQUARE],
            [
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Sand
            ]
        );
    }

    #[test]
    fn neighbor_back() {
        assert_eq!(
            BLOCKS[(SIZE_CUBE - SIZE_SQUARE)..],
            [
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::Air,
                Block::Stone,
                Block::SandStone,
            ]
        );
    }
}
