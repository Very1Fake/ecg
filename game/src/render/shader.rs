use std::borrow::Cow;

use wgpu::{Device, ShaderModule, ShaderModuleDescriptor};

// TODO: Make dynamic shader loading (at runtime)
/// Consts for declaring shaders
pub trait Shader {
    const DESCRIPTOR: ShaderModuleDescriptor<'static>;

    fn init(device: &Device) -> ShaderModule {
        device.create_shader_module(Self::DESCRIPTOR)
    }
}

/// Stores all shaders
pub struct ShaderModules {
    pub terrain: ShaderModule,
    pub figure: ShaderModule,
}

impl ShaderModules {
    pub fn init_all(device: &Device) -> Self {
        Self {
            terrain: TerrainShader::init(device),
            figure: FigureShader::init(device),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Pipeline Shaders
////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: Load shaders from assets

/// Terrain pipeline shader
pub struct TerrainShader;

impl Shader for TerrainShader {
    const DESCRIPTOR: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/shaders/terrain.wgsl"
        ))),
    };
}

/// Figure pipeline shader
pub struct FigureShader;

impl Shader for FigureShader {
    const DESCRIPTOR: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../../assets/shaders/figure.wgsl"
        ))),
    };
}
