use std::time::Duration;

use wgpu::BufferUsages;

use crate::{
    render::{
        buffer::{Buffer, DynamicBuffer},
        pipelines::{GlobalModel, Globals, GlobalsBindGroup},
        primitives::{
            instance::{Instance, RawInstance},
            vertex::Vertex,
        },
        renderer::{drawer::FirstPassDrawer, Renderer},
    },
    types::{Float32x3, Rotation},
};

use self::{
    camera::{Camera, CameraController},
    figure::voxel::Voxel,
};

pub mod camera;
pub mod figure;

// FIX: Make implement PlayState to handle events
/// Represents a world scene state
pub struct Scene {
    // Render
    pub model: GlobalModel,
    pub globals_bind_group: GlobalsBindGroup,

    // Camera
    pub camera: Camera,
    pub camera_controller: CameraController,

    // Objects
    pub pyramid_vertices: Buffer<Vertex>,
    pub pyramid_indices: Buffer<u16>,
    pub voxel: Voxel,
    pub voxel_instance: Instance,
    pub voxel_instance_buffer: DynamicBuffer<RawInstance>,
}

impl Scene {
    /// Create new `Scene`
    pub fn new(renderer: &mut Renderer) -> Self {
        let resolution = renderer.resolution();

        let model = GlobalModel {
            globals: renderer.create_consts(&[Globals::default()]),
        };

        let globals_bind_group = renderer.bind_globals(&model);

        let voxel_instance = Instance::new(Float32x3::ZERO, Rotation::IDENTITY);
        let voxel_instance_buffer = DynamicBuffer::new(&renderer.device, 1, BufferUsages::VERTEX);
        voxel_instance_buffer.update(&renderer.queue, &[voxel_instance.as_raw()], 0);

        Self {
            model,
            globals_bind_group,

            camera: Camera::new(resolution.x as f32 / resolution.y as f32),
            camera_controller: CameraController::default(),

            pyramid_vertices: Buffer::new(&renderer.device, Vertex::PYRAMID, BufferUsages::VERTEX),
            pyramid_indices: Buffer::new(&renderer.device, Vertex::INDICES, BufferUsages::INDEX),

            voxel: Voxel::new(&renderer.device),
            voxel_instance,
            voxel_instance_buffer,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer, tick_dur: Duration) {
        // Update camera
        self.camera_controller
            .update_camera(&mut self.camera, tick_dur);
        renderer.update_consts(
            &self.model.globals,
            &[Globals::new(self.camera.proj_mat(), self.camera.view_mat())],
        );

        // Update voxel position
        self.voxel_instance.position = self.camera.target;
        renderer
            .update_dynamic_buffer(&self.voxel_instance_buffer, &[self.voxel_instance.as_raw()]);
    }

    /// Draw in-game objects
    pub fn draw<'a>(&'a self, mut drawer: FirstPassDrawer<'a>) {
        // Draw "terrain"
        drawer.draw_pyramid(&self.pyramid_vertices, &self.pyramid_indices);

        // Draw figures
        drawer.draw_figure(&self.voxel, &self.voxel_instance_buffer);
    }
}
