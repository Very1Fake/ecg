use anyhow::Result;
use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, PresentMode,
};
use winit::{dpi::PhysicalSize, window::Window};

/// Handler for GPU device connection
pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
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
        // TODO: Handle errors better
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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
            // TODO: Handle case when there's no supported formats
            format: surface.get_supported_formats(&adapter)[0],
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

        Ok(Self {
            device,
            queue,
            surface,
            config,
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
