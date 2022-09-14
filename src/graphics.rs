use anyhow::{bail, Result};
use thiserror::Error;
use tracing::error;
use wgpu::{
    include_wgsl, Backends, BlendState, ColorTargetState, ColorWrites, Device, DeviceDescriptor,
    Face, Features, FragmentState, FrontFace, Instance, Limits, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    Surface, SurfaceConfiguration, TextureUsages, VertexState,
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Supported adapters not found")]
    AdapterNotFound,
    #[error("Adapter doesn't have compatible surface format")]
    CompatibleSurfaceFormatNotFound,
}

/// Handler for GPU device connection
pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // Move to separate struct
    pub render_pipeline: RenderPipeline,
}

impl Graphics {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();

        // Create new API instance (Primary APIs: Vulkan, DX12, Metal)
        let instance = Instance::new(Backends::PRIMARY);
        // Unsafe, because we use raw window handle between winit and wgpu
        let surface = unsafe { instance.create_surface(window) };

        // Request handle to physical graphical adapter
        let adapter = match instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(adapter) => adapter,
            None => {
                error!("Supported adapters not found");
                bail!(GraphicsError::AdapterNotFound)
            }
        };

        // device: connection to graphic device
        // queue: commands buffer
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    // TODO: Decide wether to support WASM target or not
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_supported_formats(&adapter).get(0) {
                Some(format) => *format,
                None => {
                    error!("Adapter doesn't have compatible surface format");
                    bail!(GraphicsError::CompatibleSurfaceFormatNotFound)
                }
            },
            width: size.width,
            height: size.height,
            // Rendering mode
            // - Immediate
            // - Fifo: VSync
            // - RelaxedFifo: Adaptive Sync (AMD on Vulkan)
            // - Mailbox: GSync (DX11/12 or NVIDIA on Vulkan)
            // TODO: Add support for switching modes in game settings
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // Load and compile our shader
        let shader = device.create_shader_module(include_wgsl!("../assets/shaders/shader.wgsl"));

        // Create pipeline bind groups layout
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        // Create a handle to render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Vertex shader entry point
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            // Properties of pipeline at primitives assembly and rasterization
            primitive: PrimitiveState {
                // Use vertices as triangles
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                // Used for example to draw wireframes
                // Requires `NON_FILL_POLYGON_MODE` feature from GPU device
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            // No depth yet
            depth_stencil: None,
            multisample: MultisampleState {
                // 1 to disable MSAA
                count: 1,
                mask: !0,
                // Something about anti-aliasing
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
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

        Ok(Self {
            device,
            queue,
            surface,
            config,
            size,
            render_pipeline,
        })
    }

    pub fn resize(&mut self, new: PhysicalSize<u32>) {
        self.size = new;
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config)
    }

    pub fn recover_surface(&mut self) {
        self.resize(self.size)
    }
}
