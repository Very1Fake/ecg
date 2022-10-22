use anyhow::{bail, Result};
use thiserror::Error;
use tracing::error;
use wgpu::{
    Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages,
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
    pub supported_surface: TextureFormat,
    pub size: PhysicalSize<u32>,
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

        Ok(Self {
            device,
            queue,
            surface,
            config,
            supported_surface,
            size,
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
