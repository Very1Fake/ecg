use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device,
    Face, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, StencilState, SurfaceConfiguration, VertexState,
};

use crate::render::{
    primitives::{instance::RawInstance, vertex::Vertex},
    shader::ShaderStore,
    texture::Texture,
};

pub struct FigurePipeline {
    pub pipeline: RenderPipeline,
    pub layout: PipelineLayout,
}

impl FigurePipeline {
    pub const LAYOUT: BindGroupLayoutEntry = BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };

    pub fn new<'a>(
        device: &Device,
        config: &SurfaceConfiguration,
        shader: &ShaderStore,
        bind_layouts: &'a [&'a BindGroupLayout],
    ) -> Self {
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("PipelineLayout: Figure"),
            bind_group_layouts: bind_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RenderPipeline: Figure"),
            layout: Some(&layout),
            // Vertex shader entry point
            vertex: VertexState {
                module: &shader.0,
                entry_point: "vs_main",
                buffers: &[Vertex::LAYOUT, RawInstance::LAYOUT],
            },
            // Properties of pipeline at primitives assembly and rasterization
            primitive: PrimitiveState {
                // Use vertices as triangles
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                // Used for example to draw wireframes
                // Requires `NON_FILL_POLYGON_MODE` feature from GPU device
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            // No depth yet
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                // 1 to disable MSAA
                count: 1,
                mask: !0,
                // Something about anti-aliasing
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: "fs_main",
                // Color output formats. Just set to surface format
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self { pipeline, layout }
    }
}
