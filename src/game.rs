use std::{iter::once, time::Instant};

use anyhow::{Context, Result};
use bytemuck::cast_slice;
use tracing::{debug, debug_span, info};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Fullscreen,
};

#[cfg(feature = "debug_ui")]
use crate::egui::{DebugPayload, DebugUI};

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
    types::{Event, Float32x3, Rotation},
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

    // Debug UI
    #[cfg(feature = "debug_ui")]
    pub debug_ui: DebugUI,

    // Rendering related
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_bind: CameraBind,

    // Objects
    pub voxel: Voxel,
    pub voxel_instance: Instance,

    // In-game related
    pub start_time: Instant,
    pub last_tick: Instant,

    // UI related
    pub paused: bool,
}

impl Game {
    pub fn new(window: &Window, graphics: Graphics) -> Self {
        // Logging span
        let _span = debug_span!("game_init").entered();
        let now = Instant::now();

        info!("Creating new game instance");

        let camera = {
            let size = window.inner.inner_size();
            Camera::new(size.width, size.height)
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

        #[cfg(feature = "debug_ui")]
        let debug_ui = {
            info!("Initializing debug UI");
            DebugUI::new(&window.inner, &graphics)
        };

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
            #[cfg(feature = "debug_ui")]
            debug_ui,
            camera,
            camera_controller: CameraController::default(),
            camera_bind,
            voxel,
            voxel_instance,
            start_time: now,
            last_tick: now,
            paused: false,
        }
    }

    /// Processes pause/resume events
    pub fn pause(&mut self, paused: bool, window: &mut Window) -> Result<()> {
        if paused {
            self.paused = true;
            self.camera_controller.reset();
        } else {
            self.paused = false;
        }

        window.grab_cursor(!self.paused)?;

        Ok(())
    }

    /// Handles window resize event
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
        // See: https://github.com/rust-windowing/winit/issues/208
        if size.width > 0 && size.height > 0 {
            self.graphics.resize(size);
            self.camera.proj_resize(size.width, size.height);

            // Recreate depth texture with new surface size
            self.depth_texture = Texture::new_depth(
                &self.graphics.device,
                &self.graphics.config,
                "Depth Texture",
            );
        }
    }

    // TODO: Use `winit_input_helper`
    pub fn input(&mut self, event: Event, control_flow: &mut ControlFlow, window: &mut Window) {
        #[cfg(feature = "debug_ui")]
        // Let debug UI handle occurred event, if cursor detached from camera
        self.debug_ui.event(&event, self.paused);

        // Update
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
                    match (key, state) {
                        // Close game
                        (VirtualKeyCode::Escape, ElementState::Pressed) => control_flow.set_exit(),
                        // Pause/Resume game
                        (VirtualKeyCode::P, ElementState::Pressed) => {
                            // FIX: Proper error handling
                            self.pause(!self.paused, window).unwrap()
                        }
                        // Toggle debug UI
                        #[cfg(feature = "debug_ui")]
                        (VirtualKeyCode::F3, ElementState::Released) => {
                            self.debug_ui.enabled = !self.debug_ui.enabled
                        }
                        // Toggle fullscreen mode
                        (VirtualKeyCode::F11, ElementState::Pressed) => {
                            match window.inner.fullscreen() {
                                Some(_) => {
                                    debug!("Switching to windowed mode");
                                    window.inner.set_fullscreen(None)
                                }
                                None => {
                                    // Available fullscreen modes for primary monitor
                                    let mut modes = window
                                        .inner
                                        .primary_monitor()
                                        .context("Can't identify primary monitor")
                                        .unwrap()
                                        .video_modes()
                                        .collect::<Vec<_>>();

                                    // Sort modes by size
                                    modes.sort_by_cached_key(|mode| {
                                        let size = mode.size();
                                        size.height * size.width
                                    });

                                    let mode = modes
                                        .last()
                                        .context("Proper fullscreen mode not found")
                                        .unwrap();

                                    debug!(
                                        size = ?mode.size(),
                                        bit_depth = mode.bit_depth(),
                                        refresh_rate_millihertz = mode.refresh_rate_millihertz(),
                                        "Switching to exclusive fullscreen mode"
                                    );
                                    window
                                        .inner
                                        .set_fullscreen(Some(Fullscreen::Exclusive(mode.clone())));
                                }
                            }
                        }
                        _ => {}
                    }

                    if !self.paused {
                        self.camera_controller.virtual_key(key, state);
                    }
                }
                WindowEvent::Resized(size) => {
                    if !self.paused {
                        self.resize(size)
                    }
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.graphics.resize(*new_inner_size)
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if !self.paused {
                        self.camera_controller.mouse_wheel(delta)
                    }
                }
                WindowEvent::Focused(focused) => {
                    if !focused {
                        // FIX: Proper error handling
                        self.pause(true, window).unwrap();
                    }
                }
                _ => {}
            },
            // FIX: Abnormal touchpad sensitivity
            // Mouse motion extracted from DeviceEvent to avoid
            // OS transformations (e.g. cursor acceleration)
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if !self.paused {
                    self.camera_controller.mouse_move(delta);
                }
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

        // Update render state
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

        #[cfg(feature = "debug_ui")]
        {
            // Update internal egui time (used for animations)
            self.debug_ui
                .platform
                .update_time(self.start_time.elapsed().as_secs_f64());
            // Update debug UI and apply game state changes
            self.debug_ui.update(DebugPayload {
                camera: &mut self.camera,
            });
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<()> {
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
                label: Some("CommandEncoder"),
            });

        // Draw in-game objects
        {
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
        }

        // Draw debug UI
        #[cfg(feature = "debug_ui")]
        let egui_draw = self.debug_ui.render(
            &self.graphics.device,
            &self.graphics.queue,
            &window.inner,
            &self.graphics.config,
            &view,
            &mut encoder,
        )?;

        // Submit render operations
        self.graphics.queue.submit(once(encoder.finish()));
        // Show rendered frame
        output.present();

        #[cfg(feature = "debug_ui")]
        if let Some(textures_delta) = egui_draw {
            self.debug_ui.cleanup(textures_delta)?;
        }

        // Something can reset cursor visibility
        window.apply_cursor_visibility();

        Ok(())
    }
}
