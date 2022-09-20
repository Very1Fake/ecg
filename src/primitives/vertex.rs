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
        // Top point of pyramid
        Self::new(Float32x3::new(0.0, 0.0, 0.0), Float32x3::new(1.0, 1.0, 1.0)),
        // Left near point of pyramid 
        Self::new(Float32x3::new(-0.5, -0.5, -0.5), Float32x3::new(1.0, 0.0, 0.0)),
        // Left far point of pyramid 
        Self::new(Float32x3::new(-0.5, -0.5, 0.5), Float32x3::new(0.0, 1.0, 0.0)),
        // Right near point of pyramid 
        Self::new(Float32x3::new(0.5, -0.5, 0.5), Float32x3::new(0.0, 0.0, 1.0)),
        // Right far point of pyramid
        Self::new(Float32x3::new(0.5, -0.5, 0.5), Float32x3::new(1.0, 0.0, 0.0)),
    ];

    #[rustfmt::skip]
    pub const INDICES: &'static [u16] = &[
        0, 1, 3,
        0, 4, 2,
        0, 2, 1,
        0, 3, 4,
        1, 2, 3,
        3, 2, 4,
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
