use std::time::Instant;

use anyhow::{Context, Result};
use tracing::{debug, debug_span, info};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Fullscreen,
};

#[cfg(feature = "debug_overlay")]
use crate::egui::{DebugOverlay, DebugPayload};

use crate::{
    render::renderer::Renderer,
    scene::Scene,
    types::{Event, U32x2},
    window::Window,
};

/// Game instance
pub struct Game {
    // Low API
    renderer: Renderer,

    scene: Scene,

    // Debug UI
    #[cfg(feature = "debug_overlay")]
    debug_overlay: DebugOverlay,

    // In-game related
    #[cfg(feature = "debug_overlay")]
    start_time: Instant,
    last_tick: Instant,

    // UI related
    paused: bool,
}

impl Game {
    pub fn new(window: &Window, mut renderer: Renderer) -> Self {
        // Logging span
        let _span = debug_span!("game_init").entered();
        let now = Instant::now();

        info!("Creating new game instance");

        info!("Initializing game scene");
        let scene = Scene::new(&mut renderer);

        #[cfg(feature = "debug_overlay")]
        let debug_overlay = {
            info!("Initializing debug UI");
            DebugOverlay::new(&window.inner)
        };

        // TODO: Refactor: make more structs
        // TODO: Split `Graphics` new() operations. Leave only low API initialization
        // TODO: Stopped at pipelines. All pipelines has access to all layouts???

        Self {
            renderer,
            scene,
            #[cfg(feature = "debug_overlay")]
            debug_overlay,
            #[cfg(feature = "debug_overlay")]
            start_time: now,
            last_tick: now,
            paused: false,
        }
    }

    /// Processes pause/resume events
    pub fn pause(&mut self, paused: bool, window: &mut Window) -> Result<()> {
        if paused {
            self.paused = true;
            self.scene.camera_controller.reset();
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
            self.scene.camera.proj_resize(size.width, size.height);

            self.renderer.on_resize(U32x2::new(size.width, size.height));
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
                        (VirtualKeyCode::F3, ElementState::Released) => self.debug_overlay.toggle(),
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
                        self.scene.camera_controller.virtual_key(key, state);
                    }
                }
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self
                    .renderer
                    .on_resize(U32x2::new(new_inner_size.width, new_inner_size.height)),
                WindowEvent::MouseWheel { delta, .. } => {
                    if !self.paused {
                        self.scene.camera_controller.mouse_wheel(delta)
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
                    self.scene.camera_controller.mouse_move(delta);
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

        // Update scene
        self.scene.update(&mut self.renderer, tick_dur);

        // Update debug overlay
        #[cfg(feature = "debug_overlay")]
        self.debug_overlay.update(
            self.start_time.elapsed().as_secs_f64(),
            DebugPayload {
                scene: &mut self.scene,
                renderer: &self.renderer,
            },
        );
    }

    pub fn render(&mut self, window: &Window) -> Result<()> {
        if let Some(mut drawer) = self
            .renderer
            .start_frame(&self.scene.globals_bind_group)
            .expect("Unrecoverable render error when starting a new frame")
        {
            self.scene.draw(drawer.first_pass());

            #[cfg(feature = "debug_overlay")]
            if self.debug_overlay.enabled {
                drawer.draw_debug_overlay(
                    &mut self.debug_overlay.platform,
                    window.inner.scale_factor() as f32,
                )?;
            }
        }

        // Something can reset cursor visibility
        window.apply_cursor_visibility();

        Ok(())
    }
}
