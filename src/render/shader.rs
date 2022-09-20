use std::borrow::Cow;

use wgpu::{Device, ShaderModule, ShaderModuleDescriptor};

// TODO: Make dynamic shader loading (at runtime)
/// Consts for declaring shaders
pub trait Shader {
    const DESCRIPTOR: ShaderModuleDescriptor<'static>;
}

// Stores shader module
pub struct ShaderStore(pub ShaderModule);

impl ShaderStore {
    pub fn new<T: Shader>(device: &Device) -> Self {
        Self(device.create_shader_module(T::DESCRIPTOR))
    }
}

pub struct MainShader;

impl Shader for MainShader {
    const DESCRIPTOR: ShaderModuleDescriptor<'static> = ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../../assets/shaders/shader.wgsl"
        ))),
    };
}
