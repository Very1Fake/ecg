// TODO: Make crate from this module

use std::time::Instant;

use common::clock::ClockStats;
use egui::{
    global_dark_light_mode_switch, Context, FontDefinitions, Grid, RadioButton, Slider, Style,
    TopBottomPanel, Window,
};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{
    render::renderer::Renderer,
    scene::{
        camera::{Camera, CameraMode},
        Scene,
    },
    types::WEvent,
};

/// Handles everything related to debug overlay drawing
pub struct DebugOverlay {
    // Inner state
    pub platform: Platform,
    state: DebugOverlayState,
    time: Instant,
}

impl DebugOverlay {
    pub fn new(window: &WinitWindow) -> Self {
        let size = window.inner_size();

        Self {
            platform: Platform::new(PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Style::default(),
            }),
            state: DebugOverlayState::new(),
            time: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &WEvent, cursor_grubbed: bool) -> bool {
        if let WEvent::WindowEvent {
            event: window_event,
            ..
        } = &event
        {
            match window_event {
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    self.platform.handle_event(event)
                }
                WindowEvent::ReceivedCharacter(_)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorEntered { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::Touch(_)
                    if !cursor_grubbed =>
                {
                    self.platform.handle_event(event)
                }
                _ => {}
            }
        }

        self.platform.captures_event(event)
    }

    pub fn toggle_top_bar(&mut self) {
        self.state.top_bar_visible = !self.state.top_bar_visible;
    }

    pub fn update(&mut self, payload: DebugPayload) {
        // Update internal egui time (used for animations)
        self.platform.update_time(self.time.elapsed().as_secs_f64());

        // Begin frame
        self.platform.begin_frame();

        // Draw UI
        self.state.draw(&self.platform.context(), payload);
    }
}

pub struct DebugPayload<'a> {
    pub clock_stats: ClockStats,
    pub scene: &'a mut Scene,
    pub renderer: &'a Renderer,
}

/// Represents debug overlay state (windows, buttons, etc.)
pub struct DebugOverlayState {
    /// Overlay top bar
    pub top_bar_visible: bool,
    /// gpu timings
    wgpu_profiler_opened: bool,
    /// Camera tracker window
    camera_tracker_opened: bool,
}

impl DebugOverlayState {
    pub const fn new() -> Self {
        Self {
            top_bar_visible: true,
            wgpu_profiler_opened: false,
            camera_tracker_opened: false,
        }
    }

    // TODO: Shift+F3 shortcut to hide menu_bar
    pub fn draw(&mut self, ctx: &Context, payload: DebugPayload) {
        let DebugPayload {
            clock_stats,
            scene: Scene { camera, .. },
            renderer,
        } = payload;

        if self.top_bar_visible {
            TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    global_dark_light_mode_switch(ui);
                    ui.separator();
                    ui.menu_button("Game", |menu| {
                        if menu.button("wgpu Profiler").clicked() {
                            self.wgpu_profiler_opened = true;
                        }
                    });
                    ui.menu_button("Scene", |menu| {
                        if menu.button("Camera").clicked() {
                            self.camera_tracker_opened = true;
                        }
                        if menu.button("Reset").clicked() {
                            match &mut camera.mode {
                                CameraMode::FirstPerson { forward } => {
                                    *forward = CameraMode::DEFAULT_FORWARD
                                }
                                CameraMode::ThirdPerson { target, distance } => {
                                    *target = CameraMode::DEFAULT_TARGET;
                                    *distance = CameraMode::DEFAULT_DISTANCE;
                                }
                            }
                            camera.yaw = Camera::DEFAULT_YAW.to_radians();
                            camera.pitch = Camera::DEFAULT_PITCH.to_radians();
                        }
                    });
                    ui.separator();
                    ui.label(format!(
                        "FPS: {:.1} ({}ms)",
                        clock_stats.avg_tps,
                        clock_stats.avg_tick_dur.as_millis(),
                    ));
                })
            });
        }

        Window::new("wgpu Profiler")
            .open(&mut self.wgpu_profiler_opened)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("wgpu Backend: {}", renderer.graphics_backend(),));
                ui.collapsing("Timings", |ui| {
                    renderer.timings().iter().for_each(|timing| {
                        ui.label(format!(
                            "{0:1$}{2}: {3:.3}ms",
                            ' ',
                            timing.0 as usize + 1,
                            timing.1,
                            timing.2 * 1000.0
                        ));
                    });
                });
            });

        Window::new("Camera")
            .open(&mut self.camera_tracker_opened)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.collapsing("Tweaks", |ui| {
                    Grid::new("camera_tweaks")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Mode");
                            ui.vertical(|ui| {
                                if ui
                                    .add(RadioButton::new(
                                        matches!(camera.mode, CameraMode::FirstPerson { .. }),
                                        "First Person",
                                    ))
                                    .clicked()
                                {
                                    camera.mode = CameraMode::first_person();
                                }
                                if ui
                                    .add(RadioButton::new(
                                        matches!(camera.mode, CameraMode::ThirdPerson { .. }),
                                        "Third Person",
                                    ))
                                    .clicked()
                                {
                                    camera.mode = CameraMode::ThirdPerson {
                                        target: camera.position,
                                        distance: CameraMode::DEFAULT_DISTANCE,
                                    };
                                }
                            });
                            ui.end_row();

                            ui.label("FOV");
                            ui.add(
                                Slider::new(&mut camera.fov, Camera::MIN_FOV..=Camera::MAX_FOV)
                                    .custom_formatter(|fov, _| {
                                        format!("{:.1}° ({:.2})", fov.to_degrees(), fov)
                                    }),
                            );
                            ui.end_row();

                            ui.label("Z Far");
                            ui.add(
                                Slider::new(&mut camera.far, Camera::MIN_Z_FAR..=Camera::MAX_Z_FAR)
                                    .max_decimals(1),
                            );
                            ui.end_row();
                        });
                });
                ui.collapsing("Tracker", |ui| {
                    ui.label(format!(
                        "Position: x:{:.3} y:{:.3} z:{:.3}\n\
                        Yaw: {:.3} ({:.2})\n\
                        Pitch: {:.3} ({:.2})\n\
                        FOV: {:.3} {:.2}\n\
                        {:#?}",
                        camera.position.x,
                        camera.position.y,
                        camera.position.z,
                        camera.yaw,
                        camera.yaw.to_degrees(),
                        camera.pitch,
                        camera.pitch.to_degrees(),
                        camera.fov,
                        camera.fov.to_degrees(),
                        camera.mode
                    ));
                });
            });
    }
}
