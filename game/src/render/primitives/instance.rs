use core::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

use crate::{
    render::buffer::Bufferable,
    types::{F32x3, Matrix4, Rotation},
};

/// Represents instance options
pub struct Instance {
    // Position of the instance in the world
    pub position: F32x3,
    // Rotation of the instance
    pub rotation: Rotation,
}

impl Bufferable for Instance {
    const LABEL: &'static str = "InstanceBuffer";
}

impl Instance {
    pub fn new(position: F32x3, rotation: Rotation) -> Self {
        Self { position, rotation }
    }

    pub fn as_raw(&self) -> RawInstance {
        RawInstance {
            model: Matrix4::from_translation(self.position),
        }
    }
}

/// Container for trans
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct RawInstance {
    model: Matrix4,
}

impl RawInstance {
    pub const ATTRS: [VertexAttribute; 4] =
        vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &Self::ATTRS,
    };
}

impl Bufferable for RawInstance {
    const LABEL: &'static str = "InstanceBuffer";
}
