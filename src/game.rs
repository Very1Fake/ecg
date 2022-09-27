use std::{iter::once, time::Instant};

use bytemuck::cast_slice;
use tracing::{debug_span, info};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    SurfaceError, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::{
    graphics::Graphics,
    render::{
        model::DrawModel,
        pipeline::{figure::FigurePipeline, terrain::TerrainPipeline},
        primitives::{instance::Instance, vertex::Vertex},
        shader::{FigureShader, ShaderStore, TerrainShader},
        texture::Texture,
    },
    scene::{
        camera::{Camera, CameraBind, CameraController},
        figure::voxel::Voxel,
    },
    types::{Float32x3, Rotation},
    window::Window,
};

/// Game instance
pub struct Game {
    // Low API
    pub graphics: Graphics,

    // WGPU related
    pub shaders: (ShaderStore, ShaderStore),
    pub terrain_pipeline: TerrainPipeline,
    pub figure_pipeline: FigurePipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub depth_texture: Texture,

    // Rendering related
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_bind: CameraBind,

    // Objects
    pub voxel: Voxel,
    pub voxel_instance: Instance,

    // In-game related
    pub last_tick: Instant,
}

impl Game {
    pub fn new(window: &Window, graphics: Graphics) -> Self {
        // Logging span
        let _span = debug_span!("game_init").entered();

        info!("Creating new game instance");

        let camera = {
            let size = window.inner.inner_size();
            Camera::new(
                Float32x3::new(0.0, 0.5, 5.0),
                Float32x3::ZERO,
                size.width,
                size.height,
                45.0,
                0.1,
                100.0,
            )
        };
        info!("Creating camera binds");
        let camera_bind = CameraBind::new(&graphics.device, &camera);

        // Shader hardcoded to game binary
        let shaders = {
            info!("Loading terrain shader module");
            let terrain = ShaderStore::new::<TerrainShader>(&graphics.device);
            info!("Loading figure shader module");
            let figure = ShaderStore::new::<FigureShader>(&graphics.device);
            (terrain, figure)
        };

        let voxel_instance = Instance::new(Float32x3::ZERO, Rotation::IDENTITY);

        info!("Creating terrain pipeline");
        let terrain_pipeline = TerrainPipeline::new(
            &graphics.device,
            &graphics.config,
            &shaders.0,
            &[&camera_bind.layout],
        );

        // TODO: Make container for buffers
        info!("Creating figure pipeline");
        let figure_pipeline = FigurePipeline::new(
            &graphics.device,
            &graphics.config,
            &shaders.1,
            &[&camera_bind.layout],
        );

        info!("Creating vertex buffer");
        let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(Vertex::PYRAMID),
            usage: BufferUsages::VERTEX,
        });

        info!("Creating index buffer");
        let index_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(Vertex::INDICES),
            usage: BufferUsages::INDEX,
        });

        info!("Creating instance buffer");
        let instance_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Primary instance buffer"),
            contents: bytemuck::cast_slice(&[voxel_instance.as_raw()]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let depth_texture = Texture::new_depth(&graphics.device, &graphics.config, "Depth Texture");

        let voxel = Voxel::new(&graphics.device);

        // TODO: Refactor: make more structs
        // TODO: Split `Graphics` new() operations. Leave only low API initialization
        // TODO: Stopped at pipelines. All pipelines has access to all layouts???

        Self {
            graphics,
            shaders,
            terrain_pipeline,
            figure_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            depth_texture,
            camera,
            camera_controller: CameraController::default(),
            camera_bind,
            voxel,
            voxel_instance,
            last_tick: Instant::now(),
        }
    }

    #[inline]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.graphics.resize(size);
        self.camera.proj_resize(size.width, size.height);

        // Recreate depth texture with new surface size
        self.depth_texture = Texture::new_depth(
            &self.graphics.device,
            &self.graphics.config,
            "Depth Texture",
        );
    }

    pub fn input(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    match key {
                        // Close game
                        VirtualKeyCode::Escape if matches!(state, ElementState::Released) => {
                            control_flow.set_exit()
                        }
                        _ => {}
                    }

                    self.camera_controller.virtual_key(key, state);
                }
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.graphics.resize(*new_inner_size)
                }
                WindowEvent::MouseWheel { delta, .. } => self.camera_controller.mouse_wheel(delta),
                _ => {}
            },
            // FIX: Abnormal touchpad sensitivity
            // Mouse motion extracted from DeviceEvent to avoid
            // OS transformations (e.g. cursor acceleration)
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.camera_controller.mouse_move(delta);
            }
            _ => {}
        }
    }

    /// Update game state
    pub fn update(&mut self) {
        // Simple tick counter
        // FIX: Make better ticking system
        let tick = Instant::now();
        let tick_dur = tick - self.last_tick;
        self.last_tick = tick;

        self.camera_controller
            .update_camera(&mut self.camera, tick_dur);
        self.camera_bind
            .update_buffer(&self.graphics.queue, &self.camera.uniform());

        // Update voxel instance position
        self.voxel_instance.position = self.camera.target;
        self.graphics.queue.write_buffer(
            &self.instance_buffer,
            0,
            cast_slice(&[self.voxel_instance.as_raw()]),
        );
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        // Texture for drawing (frame)
        let output = self.graphics.surface.get_current_texture()?;

        // View for texture, required to control how the rendering code interacts with the texture
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Used to send series of operations to GPU
        let mut encoder = self
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Pipe Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            // Where to we draw colors
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    // Where to pick the previous frame.
                    // Clears screen with specified color
                    // NOTE: Right now used as simple skybox
                    load: LoadOp::Clear(Color {
                        r: 0.458,
                        g: 0.909,
                        b: 1.0,
                        a: 1.0,
                    }),
                    // Write results to texture
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        // Draw "terrain"
        render_pass.set_pipeline(&self.terrain_pipeline.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..Vertex::INDICES.len() as u32, 0, 0..1);

        // Draw figures
        render_pass.set_pipeline(&self.figure_pipeline.pipeline);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_model(&self.voxel, &self.camera_bind);

        drop(render_pass);

        // Submit render operations
        self.graphics.queue.submit(once(encoder.finish()));
        // Show rendered frame
        output.present();

        Ok(())
    }
}
