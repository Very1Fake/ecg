use bytemuck::cast_slice;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, IndexFormat,
};

use crate::{
    render::{
        model::Model,
        primitives::{direction::Direction, quad::Quad, vertex::Vertex},
    },
    types::F32x3,
};

pub struct Voxel {
    pub vertices: Buffer,
    pub indices: Buffer,
    pub indices_count: u32,
}

impl Voxel {
    pub fn new(device: &Device) -> Self {
        let vertices: Vec<Vertex> = Direction::ALL
            .into_iter()
            .flat_map(|dir| {
                Quad::new(dir, F32x3::ZERO)
                    .corners()
                    .into_iter()
                    .map(|position| Vertex {
                        // Rescale
                        position: position * 0.1,
                        color: F32x3::ZERO,
                    })
            })
            .collect();

        let indices: Vec<u16> = (0..vertices.len() as u16)
            .step_by(4)
            .flat_map(|i| [i, i + 1, i + 2, i, i + 2, i + 3])
            .collect();

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("ModelVertex: Voxel"),
            contents: cast_slice(vertices.as_slice()),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("ModelIndex: Voxel"),
            contents: cast_slice(indices.as_slice()),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertices: vertex_buffer,
            indices: index_buffer,
            indices_count: indices.len() as u32,
        }
    }
}

impl Model for Voxel {
    const INDEX_FORMAT: IndexFormat = IndexFormat::Uint16;

    fn get_vertices(&self) -> &Buffer {
        &self.vertices
    }

    fn get_indices(&self) -> (&Buffer, u32) {
        (&self.indices, self.indices_count)
    }
}
