// TODO: Make crate from this module

use egui::{
    global_dark_light_mode_switch, Context, FontDefinitions, Style, TopBottomPanel, Window,
};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{
    render::renderer::Renderer,
    scene::{
        camera::{Camera, CameraMode},
        Scene,
    },
    types::Event,
};

/// Handles everything related to debug overlay drawing
pub struct DebugOverlay {
    // Inner state
    pub enabled: bool,
    pub platform: Platform,
    state: DebugOverlayState,
}

impl DebugOverlay {
    pub fn new(window: &WinitWindow) -> Self {
        let size = window.inner_size();

        Self {
            enabled: false,
            platform: Platform::new(PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Style::default(),
            }),
            state: DebugOverlayState::default(),
        }
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled
    }

    pub fn event(&mut self, event: &Event, paused: bool) {
        if self.enabled {
            if let Event::WindowEvent {
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
                        if paused =>
                    {
                        self.platform.handle_event(event)
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn update(&mut self, elapsed: f64, payload: DebugPayload) {
        // Update internal egui time (used for animations)
        self.platform.update_time(elapsed);

        if self.enabled {
            // Begin frame
            self.platform.begin_frame();

            // Draw UI
            self.state.draw(&self.platform.context(), payload);
        }
    }
}

pub struct DebugPayload<'a> {
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
                            CameraMode::ThirdPerson { distance } => {
                                *distance = CameraMode::DEFAULT_DISTANCE
                            }
                        }
                        payload.scene.camera.target = Camera::DEFAULT_TARGET;
                        payload.scene.camera.yaw = Camera::DEFAULT_YAW.to_radians();
                        payload.scene.camera.pitch = Camera::DEFAULT_PITCH.to_radians();
                    }
                })
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

        Window::new("Camera Tracker")
            .open(&mut self.camera_tracker_opened)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Position: x:{:.3} y:{:.3} z:{:.3}\n\
                    Target: x:{:.3} y:{:.3} z:{:.3}\n\
                    Yaw: {:.3} ({:.2})\n\
                    Pitch: {:.3} ({:.2})",
                    payload.scene.camera.position.x,
                    payload.scene.camera.position.y,
                    payload.scene.camera.position.z,
                    payload.scene.camera.target.x,
                    payload.scene.camera.target.y,
                    payload.scene.camera.target.z,
                    payload.scene.camera.yaw,
                    payload.scene.camera.yaw.to_degrees(),
                    payload.scene.camera.pitch,
                    payload.scene.camera.pitch.to_degrees(),
                ));
                ui.label(format!("{:?}", payload.scene.camera.mode))
            });
    }
}
