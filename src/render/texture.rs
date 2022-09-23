use tracing::{debug, debug_span};
use wgpu::{
    AddressMode, CompareFunction, Device, Extent3d, FilterMode, Sampler, SamplerDescriptor,
    SurfaceConfiguration, Texture as WTexture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

/// Represents image that has been uploaded to the GPU
pub struct Texture {
    pub texture: WTexture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub size: Extent3d,
    pub format: TextureFormat,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new_depth(device: &Device, config: &SurfaceConfiguration, label: &str) -> Self {
        let _span = debug_span!("new_depth_texture");

        let size = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        debug!(texture = label, "Creating new depth texture");
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        debug!(texture = label, "Creating new sampler");
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(CompareFunction::LessEqual),
            anisotropy_clamp: None,
            border_color: None,
        });

        Self {
            texture,
            view,
            sampler,
            size,
            format: Self::DEPTH_FORMAT,
        }
    }
}
