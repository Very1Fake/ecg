use std::time::Instant;

use anyhow::{Context, Result};
use bytemuck::cast_slice;
use tracing::{debug, debug_span, info};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Fullscreen,
};

#[cfg(feature = "debug_overlay")]
use crate::egui::{DebugOverlay, DebugPayload};

use crate::{
    graphics::Graphics,
    render::{
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

    // Debug UI
    #[cfg(feature = "debug_overlay")]
    pub debug_overlay: DebugOverlay,

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

        #[cfg(feature = "debug_overlay")]
        let debug_overlay = {
            info!("Initializing debug UI");
            DebugOverlay::new(&window.inner, &graphics)
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
            #[cfg(feature = "debug_overlay")]
            debug_overlay,
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
            self.graphics.depth_texture = Texture::new_depth(
                &self.graphics.device,
                &self.graphics.config,
                "Depth Texture",
            );
        }
    }

    // TODO: Use `winit_input_helper`
    pub fn input(&mut self, event: Event, control_flow: &mut ControlFlow, window: &mut Window) {
        #[cfg(feature = "debug_overlay")]
        // Let debug UI handle occurred event, if cursor detached from camera
        self.debug_overlay.event(&event, self.paused);

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
                        #[cfg(feature = "debug_overlay")]
                        (VirtualKeyCode::F3, ElementState::Released) => {
                            self.debug_overlay.enabled = !self.debug_overlay.enabled
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

        #[cfg(feature = "debug_overlay")]
        {
            // Update internal egui time (used for animations)
            self.debug_overlay
                .platform
                .update_time(self.start_time.elapsed().as_secs_f64());
            // Update debug UI and apply game state changes
            self.debug_overlay.update(DebugPayload {
                camera: &mut self.camera,
            });
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<()> {
        #[cfg(feature = "debug_overlay")]
        let mut egui_draw = None;

        if let Some(mut drawer) = self
            .graphics
            .start_frame()
            .expect("Unrecoverable render error when starting a new frame")
        {
            // Draw in-game objects
            {
                let mut first_pass = drawer.first_pass();

                // Draw "terrain"
                first_pass.draw_pyramid(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    &self.terrain_pipeline,
                    &self.camera_bind,
                );

                // Draw figures
                first_pass.draw_figures(
                    &self.voxel,
                    &self.instance_buffer,
                    &self.figure_pipeline,
                    &self.camera_bind,
                );
            }

            #[cfg(feature = "debug_overlay")]
            if self.debug_overlay.enabled {
                egui_draw = drawer.draw_debug_overlay(
                    &mut self.debug_overlay.render_pass,
                    &mut self.debug_overlay.platform,
                    window.inner.scale_factor() as f32,
                )?;
            }
        }

        #[cfg(feature = "debug_overlay")]
        if let Some(textures_delta) = egui_draw {
            self.debug_overlay.cleanup(textures_delta)?;
        }

        // Something can reset cursor visibility
        window.apply_cursor_visibility();

        Ok(())
    }
}
