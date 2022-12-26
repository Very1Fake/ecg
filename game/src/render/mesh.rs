use std::sync::mpsc::Sender;

use crate::render::primitives::quad::Quad;
use common::{
    block::Block,
    coord::{BlockCoord, ChunkCoord, CHUNK_SIZE},
    direction::Direction,
};
use common_log::prof;

use super::primitives::vertex::Vertex;

pub type Neighbor = Option<[Block; CHUNK_SIZE]>;
pub type MeshTaskResult = (ChunkCoord, TerrainMesh);

#[derive(Default, Debug)]
pub struct Neighbors {
    down: Option<[Block; CHUNK_SIZE]>,
    up: Option<[Block; CHUNK_SIZE]>,
    left: Option<[Block; CHUNK_SIZE]>,
    right: Option<[Block; CHUNK_SIZE]>,
    front: Option<[Block; CHUNK_SIZE]>,
    back: Option<[Block; CHUNK_SIZE]>,
}

/// Mesh builder for terrain chunks
pub struct TerrainMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn task(tx: Sender<MeshTaskResult>, coord: ChunkCoord, blocks: &[Block], neighbors: Neighbors) {
        let _ = tx.send((coord, Self::build(coord, blocks, neighbors)));
    }

    pub fn build(coord: ChunkCoord, blocks: &[Block], neighbors: Neighbors) -> Self {
        prof!("TerrainMesh::build");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index: u32 = 0;

        blocks
            .iter()
            .enumerate()
            .filter_map(|(id, block)| {
                if block.opaque() {
                    let pos = BlockCoord::from(id);
                    let g_pos = coord.to_global(&pos).as_vec();
                    let mut faces = Vec::new();

                    Direction::ALL.iter().for_each(|&dir| {
                        if pos.on_chunk_edge(dir) || !blocks[pos.neighbor(dir).flatten()].opaque() {
                            faces.push(Quad::new(dir, g_pos));
                        }
                    });

                    if !faces.is_empty() {
                        return Some((block, faces));
                    }
                }

                None
            })
            .for_each(|(block, faces)| {
                let mut block_vertices = faces
                    .into_iter()
                    .flat_map(|quad| {
                        quad.corners().into_iter().map(|position| Vertex {
                            position,
                            color: block.color(),
                        })
                    })
                    .collect::<Vec<_>>();

                indices.extend(
                    (0..block_vertices.len() as u32)
                        .step_by(4)
                        .flat_map(|mut i| {
                            i += index;
                            [i, i + 1, i + 2, i, i + 2, i + 3]
                        }),
                );

                index += block_vertices.len() as u32;

                vertices.append(&mut block_vertices);
            });

        Self { vertices, indices }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}
