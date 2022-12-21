use common_log::span;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, StencilState, SurfaceConfiguration, VertexState,
};

use crate::render::{primitives::vertex::Vertex, texture::Texture};

use super::GlobalLayout;

pub struct TerrainPipeline {
    pub inner: RenderPipeline,
}

impl TerrainPipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        shader: &ShaderModule,
        globals_layout: &GlobalLayout,
    ) -> Self {
        span!(_guard, "TerrainPipeline::new");

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("PipelineLayout: Terrain"),
            bind_group_layouts: &[&globals_layout.globals],
            push_constant_ranges: &[],
        });

        Self {
            inner: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("RenderPipeline: Terrain"),
                layout: Some(&layout),
                // Vertex shader entry point
                vertex: VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::LAYOUT],
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
                    module: shader,
                    entry_point: "fs_main",
                    // Color output formats. Just set to surface format
                    targets: &[Some(ColorTargetState {
                        format: config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            }),
        }
    }
}
