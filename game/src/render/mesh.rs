use std::sync::mpsc::Sender;

use crate::render::primitives::quad::Quad;
use common::{
    block::Block,
    coord::{BlockCoord, ChunkCoord},
    direction::Direction,
    prof,
};
use rand::{thread_rng, Rng};

use super::primitives::vertex::Vertex;

pub type MeshTaskResult = (ChunkCoord, TerrainMesh);

/// Mesh builder for terrain chunks
pub struct TerrainMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn task(tx: Sender<MeshTaskResult>, coord: ChunkCoord, blocks: &[Block]) {
        let _ = tx.send((coord, Self::build(coord, blocks)));
    }

    pub fn build(coord: ChunkCoord, blocks: &[Block]) -> Self {
        prof!("TerrainMesh::build");

        let mut rng = thread_rng();
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
                let mut color = block.color();
                color.x = rng.gen_range(color.x - 0.05..=color.x + 0.05);
                color.y = rng.gen_range(color.y - 0.05..=color.y + 0.05);
                color.z = rng.gen_range(color.z - 0.05..=color.z + 0.05);

                let mut block_vertices = faces
                    .into_iter()
                    .flat_map(|quad| {
                        quad.corners()
                            .into_iter()
                            .map(|position| Vertex { position, color })
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
