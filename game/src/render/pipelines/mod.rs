use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Device, ShaderStages,
};

use crate::{
    test_buffer_align,
    types::{Matrix4, RawMatrix4},
};

use super::{
    buffer::{Bufferable, Consts},
    renderer::Renderer,
};

pub mod figure;
pub mod terrain;

// TODO: Make global layout
// TODO: Make bind groups for new layout system

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Globals {
    /// Projection matrix
    proj_mat: RawMatrix4,
    /// Camera view matrix
    view_mat: RawMatrix4,
    /// proj_mat * view_mat
    all_mat: RawMatrix4,
}

impl Bufferable for Globals {
    const LABEL: &'static str = "Uniform: Globals";
}

impl Globals {
    pub fn new(proj_mat: Matrix4, view_mat: Matrix4) -> Self {
        Self {
            proj_mat: proj_mat.to_cols_array_2d(),
            view_mat: view_mat.to_cols_array_2d(),
            all_mat: (proj_mat * view_mat).to_cols_array_2d(),
        }
    }
}

impl Default for Globals {
    fn default() -> Self {
        Self::new(Matrix4::IDENTITY, Matrix4::IDENTITY)
    }
}

test_buffer_align!(Globals);

/// Global scene data
pub struct GlobalModel {
    pub globals: Consts<Globals>,
}

impl GlobalModel {
    pub fn create(renderer: &Renderer) -> Self {
        Self {
            globals: renderer.create_consts(&[Globals::default()]),
        }
    }
}

/// Represent bind group for `Globals`
pub struct GlobalsBindGroup {
    pub inner: BindGroup,
}

/// Represents created layouts on the GPU
pub struct GlobalLayout {
    pub globals: BindGroupLayout,
}

impl GlobalLayout {
    const BASE_LAYOUT_ENTRIES: &[BindGroupLayoutEntry] = &[
        // Globals uniform
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];

    const BASE_LAYOUT_DESC: BindGroupLayoutDescriptor<'static> = BindGroupLayoutDescriptor {
        label: Some("BindGroupLayout: Globals"),
        entries: Self::BASE_LAYOUT_ENTRIES,
    };

    pub fn new(device: &Device) -> Self {
        Self {
            globals: device.create_bind_group_layout(&Self::BASE_LAYOUT_DESC),
        }
    }

    pub fn bind_globals(&self, device: &Device, global_model: &GlobalModel) -> GlobalsBindGroup {
        GlobalsBindGroup {
            inner: device.create_bind_group(&BindGroupDescriptor {
                label: Some("BindGroup: Globals"),
                layout: &self.globals,
                entries: &[
                    // Globals uniform
                    BindGroupEntry {
                        binding: 0,
                        resource: global_model.globals.buffer().as_entire_binding(),
                    },
                ],
            }),
        }
    }
}
