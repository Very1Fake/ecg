use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

use crate::types::Float32x3;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Float32x3,
    pub color: Float32x3,
}

impl Vertex {
    #[rustfmt::skip]
    pub const PYRAMID: &'static [Self] = &[
        // Front of pyramid
        Self::new(Float32x3::new(0.0, 0.0, 0.0), Float32x3::new(1.0, 0.0, 0.0)),
        Self::new(Float32x3::new(-0.5, -0.5, -0.5), Float32x3::new(0.0, 1.0, 0.0)),
        Self::new(Float32x3::new(0.5, -0.5, -0.5), Float32x3::new(0.0, 0.0, 1.0)),
        // Back of pyramid
        Self::new(Float32x3::new(0.0, 0.0, 0.0), Float32x3::new(1.0, 0.0, 0.0)),
        Self::new(Float32x3::new(0.5, -0.5, 0.5), Float32x3::new(0.0, 0.0, 1.0)),
        Self::new(Float32x3::new(-0.5, -0.5, 0.5), Float32x3::new(0.0, 1.0, 0.0)),
        // Right of pyramid
        Self::new(Float32x3::new(0.0, 0.0, 0.0), Float32x3::new(1.0, 0.0, 0.0)),
        Self::new(Float32x3::new(0.5, -0.5, -0.5), Float32x3::new(0.0, 0.0, 1.0)),
        Self::new(Float32x3::new(-0.5, -0.5, 0.5), Float32x3::new(0.0, 1.0, 0.0)),
        // Left of pyramid
        Self::new(Float32x3::new(0.0, 0.0, 0.0), Float32x3::new(1.0, 0.0, 0.0)),
        Self::new(Float32x3::new(-0.5, -0.5, 0.5), Float32x3::new(0.0, 1.0, 0.0)),
        Self::new(Float32x3::new(0.5, -0.5, -0.5), Float32x3::new(0.0, 0.0, 1.0)),
        // Bottom of pyramid
        Self::new(Float32x3::new(0.5, -0.5, -0.5), Float32x3::new(1.0, 0.0, 0.0)),
        Self::new(Float32x3::new(-0.5, -0.5, -0.5), Float32x3::new(0.0, 1.0, 0.0)),
        Self::new(Float32x3::new(-0.5, -0.5, 0.5), Float32x3::new(0.0, 0.0, 1.0)),
        Self::new(Float32x3::new(0.5, -0.5, 0.5), Float32x3::new(0.0, 1.0, 0.0)),
    ];

    pub const ATTRS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    #[inline]
    pub const fn new(position: Float32x3, color: Float32x3) -> Self {
        Self { position, color }
    }

    pub fn layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}
