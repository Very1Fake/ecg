use common::{
    block::Block,
    coord::{BlockCoord, ChunkCoord},
    prof,
};

use crate::render::primitives::{direction::Direction, quad::Quad};

use super::primitives::vertex::Vertex;

/// Mesh builder for terrain chunks
pub struct TerrainMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn build(coord: ChunkCoord, blocks: &[Block]) -> Self {
        prof!("TerrainMesh::build");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index: u32 = 0;

        blocks
            .iter()
            .enumerate()
            .filter(|(_, &block)| block.opaque())
            .for_each(|(flat_coord, block)| {
                let pos = coord
                    .to_global(&BlockCoord::from(flat_coord as i64))
                    .as_vec();
                let mut block_vertices = Direction::ALL
                    .into_iter()
                    .flat_map(|dir| {
                        Quad::new(dir, pos)
                            .corners()
                            .into_iter()
                            .map(|position| Vertex {
                                position: position * 0.1,
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
