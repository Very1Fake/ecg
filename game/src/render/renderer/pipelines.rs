// TODO: Parallel pipelines creation

use wgpu::{Device, SurfaceConfiguration};

use crate::render::{
    pipelines::{figure::FigurePipeline, terrain::TerrainPipeline},
    shader::ShaderModules,
};

use super::layouts::Layouts;

pub struct Pipelines {
    pub terrain: TerrainPipeline,
    pub figure: FigurePipeline,
}

impl Pipelines {
    pub fn create(
        device: &Device,
        layouts: &Layouts,
        shaders: &ShaderModules,
        config: &SurfaceConfiguration,
    ) -> Self {
        Self {
            terrain: TerrainPipeline::new(device, config, &shaders.terrain, &layouts.globals),
            figure: FigurePipeline::new(device, config, &shaders.figure, &layouts.globals),
        }
    }
}
