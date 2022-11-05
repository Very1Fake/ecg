use anyhow::{bail, Result};
use bytemuck::Pod;
use tokio::runtime::Runtime;
use tracing::{error, info, warn};
use wgpu::{
    Backends, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Instance,
    Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages,
};
use winit::window::Window;

use crate::{
    render::{renderer::layouts::Layouts, texture::Texture},
    types::U32x2,
};

use super::{
    buffer::{Bufferable, Consts, DynamicBuffer},
    error::RenderError,
    pipelines::GlobalsBindGroup,
    shader::ShaderModules,
};

use {drawer::Drawer, pipelines::Pipelines};

pub mod binding;
pub mod drawer;
pub mod layouts;
pub mod pipelines;

/// Represents a render state of the entire game.
/// `Renderer` contains any state necessary to interact
/// with the GPU, along with pipeline state object (PSOs)
/// needed to render different kinds of models.
pub struct Renderer {
    // wgpu related
    pub device: Device,
    pub queue: Queue,
    surface: Surface,
    pub config: SurfaceConfiguration,

    // Inner state
    resolution: U32x2,
    is_minimized: bool,

    // Textures
    depth_texture: Texture,

    _shaders: ShaderModules,
    layouts: Layouts,
    // TODO: With a large number of pipelines, make (re)creation async
    pipelines: Pipelines,

    // Shaders
    #[cfg(feature = "debug_overlay")]
    egui_render_pass: egui_wgpu_backend::RenderPass,

    /// Backend API. Used for debug purposes
    graphics_backend: String,
}

impl Renderer {
    pub fn new(window: &Window, runtime: &Runtime) -> Result<Self> {
        let size = window.inner_size();
        // TODO: Parse backend from env
        let backend = Backends::PRIMARY;

        // Create new API instance (Primary APIs: Vulkan, DX12, Metal)
        let instance = Instance::new(backend);
        // Unsafe, because we use raw window handle between winit and wgpu
        let surface = unsafe { instance.create_surface(window) };

        let adapters = instance
            .enumerate_adapters(backend)
            .enumerate()
            .collect::<Vec<_>>();

        adapters.iter().for_each(|(id, adapter)| {
            let info = adapter.get_info();
            info!(
                ?info.name,
                ?info.vendor,
                ?info.backend,
                ?info.device,
                ?info.device_type,
                "Graphic device #{id}"
            );
        });

        // Request handle to physical graphical adapter
        // TODO: Parse adapter from env
        let adapter = runtime
            .block_on(instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .ok_or(RenderError::AdapterNotFound)?;

        let info = adapter.get_info();
        info!(
            ?info.name,
            ?info.vendor,
            ?info.backend,
            ?info.device,
            ?info.device_type,
            "Selected graphic device"
        );
        let graphics_backend = format!("{:?}", &info.backend);

        // device: connection to graphic device
        // queue: commands buffer
        let (device, queue) = runtime.block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("GraphicDevice"),
                features: adapter.features(),
                // TODO: Decide wether to support WASM target or not
                limits: Limits::default(),
            },
            None,
        ))?;

        device.on_uncaptured_error(move |err| {
            error!("{err}");
            panic!("wgpu fatal error:\n{:?}\n{:?}", err, info);
        });

        let surface_format = *surface
            .get_supported_formats(&adapter)
            .get(0)
            .ok_or(RenderError::NoCompatibleSurfaceFormat)?;
        info!("Using {surface_format:?} as surface format");

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // Rendering mode
            // - Immediate
            // - Fifo: VSync
            // - RelaxedFifo: Adaptive Sync (AMD on Vulkan)
            // - Mailbox: GSync (DX11/12 or NVIDIA on Vulkan)
            // TODO: Add support for switching modes in game settings
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let depth_texture = Texture::new_depth(&device, &config, "Depth Texture");

        let shaders = ShaderModules::init_all(&device);
        let layouts = Layouts::new(&device);
        let pipelines = Pipelines::create(&device, &layouts, &shaders, &config);

        #[cfg(feature = "debug_overlay")]
        let egui_render_pass =
            egui_wgpu_backend::RenderPass::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb, 1);

        Ok(Self {
            device,
            queue,
            surface,
            config,

            resolution: U32x2::new(size.width, size.height),
            is_minimized: false,

            depth_texture,

            layouts,
            _shaders: shaders,
            pipelines,

            #[cfg(feature = "debug_overlay")]
            egui_render_pass,

            graphics_backend,
        })
    }

    /// Get graphic backend API being used
    pub fn graphics_backend(&self) -> &str {
        &self.graphics_backend
    }

    /// Get current renderer resolution
    pub fn resolution(&self) -> U32x2 {
        self.resolution
    }

    pub fn create_consts<T: Copy + Pod + Bufferable>(&self, values: &[T]) -> Consts<T> {
        Self::create_consts_inner(&self.device, &self.queue, values)
    }

    fn create_consts_inner<T: Copy + Pod + Bufferable>(
        device: &Device,
        queue: &Queue,
        values: &[T],
    ) -> Consts<T> {
        let consts = Consts::new(device, values.len());
        consts.update(queue, values, 0);
        consts
    }

    /// Update constant buffer
    pub fn update_consts<T: Copy + Pod + Bufferable>(&self, consts: &Consts<T>, values: &[T]) {
        consts.update(&self.queue, values, 0)
    }

    // TODO: Update only models
    pub fn update_dynamic_buffer<T: Copy + Pod + Bufferable>(
        &self,
        buffer: &DynamicBuffer<T>,
        values: &[T],
    ) {
        buffer.update(&self.queue, values, 0);
    }

    /// Resize surface to match window dimensions
    pub fn on_resize(&mut self, new: U32x2) {
        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
        // See: https://github.com/rust-windowing/winit/issues/208
        // Also avoids panic on texture with size of 0,0
        if new.x != 0 && new.y != 0 {
            self.is_minimized = false;

            // Resize surface
            self.resolution = new;
            self.config.width = self.resolution.x;
            self.config.height = self.resolution.y;
            self.surface.configure(&self.device, &self.config);

            // Resize depth texture
            self.depth_texture = Texture::new_depth(&self.device, &self.config, "Depth Texture");
        } else {
            self.is_minimized = true;
        }
    }

    /// Start frame rendering and create `Drawer`
    /// If there is an intermittent issue with the surface
    /// then Ok(None) will be returned
    pub fn start_frame<'a>(
        &'a mut self,
        globals: &'a GlobalsBindGroup,
    ) -> Result<Option<Drawer<'a>>> {
        if self.is_minimized {
            return Ok(None);
        }

        // Used to send series of operations to GPU
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("FirstPassEncoder"),
            });

        // The current frame texture to draw
        let texture = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(err @ (SurfaceError::Lost | SurfaceError::Outdated)) => {
                warn!("{} Recreating surface (frame will be missed)", err);
                self.on_resize(self.resolution);
                return Ok(None);
            }
            Err(SurfaceError::Timeout) => {
                // This will be resolved on the next frame
                return Ok(None);
            }
            Err(err @ SurfaceError::OutOfMemory) => {
                // If surface lost, try to recover it by reconfiguring
                bail!(err)
            }
        };

        Ok(Some(Drawer::new(encoder, self, texture, globals)))
    }
}