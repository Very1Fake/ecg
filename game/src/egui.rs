// TODO: Make crate from this module

use std::time::Instant;

use common::clock::ClockStats;
use egui::{
    global_dark_light_mode_switch, Context, FontDefinitions, RadioButton, Style, TopBottomPanel,
    Window,
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
            state: DebugOverlayState::default(),
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
#[derive(Default)]
pub struct DebugOverlayState {
    /// Debug info
    wgpu_profiler_opened: bool,
    /// Camera tracker window
    camera_tracker_opened: bool,
}

impl DebugOverlayState {
    // TODO: Shift+F3 shortcut to hide menu_bar
    pub fn draw(&mut self, ctx: &Context, payload: DebugPayload) {
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
                    if menu.button("Camera Stats").clicked() {
                        self.camera_tracker_opened = true;
                    }
                    if menu.button("Reset").clicked() {
                        match &mut payload.scene.camera.mode {
                            CameraMode::FirstPerson { forward } => {
                                *forward = CameraMode::DEFAULT_FORWARD
                            }
                            CameraMode::ThirdPerson { target, distance } => {
                                *target = CameraMode::DEFAULT_TARGET;
                                *distance = CameraMode::DEFAULT_DISTANCE;
                            }
                        }
                        payload.scene.camera.yaw = Camera::DEFAULT_YAW.to_radians();
                        payload.scene.camera.pitch = Camera::DEFAULT_PITCH.to_radians();
                    }
                });
                ui.separator();
                ui.label(format!(
                    "FPS: {:.1} ({}ms)",
                    payload.clock_stats.avg_tps, payload.clock_stats.avg_tick_dur.as_millis(),
                ));
            })
        });

        Window::new("wgpu Profiler")
            .open(&mut self.wgpu_profiler_opened)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "wgpu Backend: {}",
                    payload.renderer.graphics_backend()
                ));
            });

        Window::new("Camera")
            .open(&mut self.camera_tracker_opened)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        if ui
                            .add(RadioButton::new(
                                matches!(payload.scene.camera.mode, CameraMode::FirstPerson { .. }),
                                "First Person",
                            ))
                            .clicked()
                        {
                            payload.scene.camera.mode = CameraMode::first_person();
                        }
                        if ui
                            .add(RadioButton::new(
                                matches!(payload.scene.camera.mode, CameraMode::ThirdPerson { .. }),
                                "Third Person",
                            ))
                            .clicked()
                        {
                            payload.scene.camera.mode = CameraMode::ThirdPerson {
                                target: payload.scene.camera.position,
                                distance: CameraMode::DEFAULT_DISTANCE,
                            };
                        }
                    })
                });
                ui.collapsing("Tracker", |ui| {
                    ui.label(format!(
                        "Position: x:{:.3} y:{:.3} z:{:.3}\n\
                        Yaw: {:.3} ({:.2})\n\
                        Pitch: {:.3} ({:.2})\n\
                        FOV: {:.3} {:.2}\n\
                        {:#?}",
                        payload.scene.camera.position.x,
                        payload.scene.camera.position.y,
                        payload.scene.camera.position.z,
                        payload.scene.camera.yaw,
                        payload.scene.camera.yaw.to_degrees(),
                        payload.scene.camera.pitch,
                        payload.scene.camera.pitch.to_degrees(),
                        payload.scene.camera.fov,
                        payload.scene.camera.fov.to_degrees(),
                        payload.scene.camera.mode
                    ));
                });
            });
    }
}
