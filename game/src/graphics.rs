use anyhow::{bail, Result};
use thiserror::Error;
use tracing::{debug_span, error, warn};
use wgpu::{
    Backends, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features,
    Instance, Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::render::{drawer::Drawer, texture::Texture};

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
    pub supported_surface: TextureFormat,
    pub size: PhysicalSize<u32>,
    pub depth_texture: Texture,
}

impl Graphics {
    pub async fn new(window: &Window) -> Result<Self> {
        let _span = debug_span!("graphics_init").entered();

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
                    features: Features::default(),
                    // TODO: Decide wether to support WASM target or not
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        let supported_surface = match surface.get_supported_formats(&adapter).get(0) {
            Some(format) => *format,
            None => {
                error!("Adapter doesn't have compatible surface format");
                bail!(GraphicsError::CompatibleSurfaceFormatNotFound)
            }
        };

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: supported_surface,
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

        Ok(Self {
            device,
            queue,
            surface,
            config,
            supported_surface,
            size,
            depth_texture,
        })
    }

    pub fn resize(&mut self, new: PhysicalSize<u32>) {
        self.size = new;
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config)
    }

    /// Start frame rendering and create `Drawer`
    /// If there is an intermittent issue with the surface
    /// then Ok(None) will be returned
    pub fn start_frame<'a>(&'a mut self) -> Result<Option<Drawer<'a>>> {
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
                self.resize(self.size);
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

        Ok(Some(Drawer::new(encoder, self, texture)))
    }
}
