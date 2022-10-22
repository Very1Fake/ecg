// TODO: Make crate from this module

use anyhow::{Context as ResultContext, Result};
use egui::{
    global_dark_light_mode_switch, Context, FontDefinitions, FullOutput, Style, TexturesDelta,
    TopBottomPanel, Window,
};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{
    graphics::Graphics,
    scene::camera::{Camera, CameraMode},
    types::Event,
};

/// Handles everything related to debug UI drawing
pub struct DebugUI {
    // Inner state
    pub enabled: bool,
    pub platform: Platform,
    pub state: DebugUIState,

    // Graphics
    pub render_pass: RenderPass,
}

impl DebugUI {
    pub fn new(window: &WinitWindow, graphics: &Graphics) -> Self {
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
            state: DebugUIState::default(),
            render_pass: RenderPass::new(&graphics.device, graphics.supported_surface, 1),
        }
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

    pub fn update(&mut self, payload: DebugPayload) {
        if self.enabled {
            // Begin frame
            self.platform.begin_frame();

            // Draw UI
            self.state.draw(&self.platform.context(), payload);
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &WinitWindow,
        surface_config: &SurfaceConfiguration,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) -> Result<Option<TexturesDelta>> {
        if self.enabled {
            // Finalize frame
            // FIX: Fixes cursor flickering, but cursor icons won't change
            let FullOutput {
                textures_delta,
                shapes,
                ..
            } = self.platform.end_frame(None);

            // Tesselate shapes
            let paint_jobs = self.platform.context().tessellate(shapes);

            let screen_descriptor = &ScreenDescriptor {
                physical_width: surface_config.width,
                physical_height: surface_config.height,
                scale_factor: window.scale_factor() as f32,
            };

            // Send textures and update buffers
            self.render_pass
                .add_textures(device, queue, &textures_delta)
                .context("While uploading UI texture")?;
            self.render_pass
                .update_buffers(device, queue, &paint_jobs, screen_descriptor);

            // Record all commands to encoder
            self.render_pass
                .execute(encoder, view, &paint_jobs, screen_descriptor, None)
                .context("While executing ui commands")?;

            Ok(Some(textures_delta))
        } else {
            Ok(None)
        }
    }

    pub fn cleanup(&mut self, textures_delta: TexturesDelta) -> Result<()> {
        self.render_pass.remove_textures(textures_delta)?;

        Ok(())
    }
}

pub struct DebugPayload<'a> {
    pub camera: &'a mut Camera,
}

/// Represents debug UI state (windows, buttons, etc.)
#[derive(Default)]
pub struct DebugUIState {
    /// Camera tracker window
    camera_tracker_opened: bool,
}

impl DebugUIState {
    // TODO: Shift+F3 shortcut to hide menu_bar
    pub fn draw(&mut self, ctx: &Context, payload: DebugPayload) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                global_dark_light_mode_switch(ui);
                ui.separator();
                ui.menu_button("Scene", |menu| {
                    if menu.button("Camera Stats").clicked() {
                        self.camera_tracker_opened = true;
                    }
                    if menu.button("Reset").clicked() {
                        match &mut payload.camera.mode {
                            CameraMode::ThirdPerson { distance } => {
                                *distance = CameraMode::DEFAULT_DISTANCE
                            }
                        }
                        payload.camera.target = Camera::DEFAULT_TARGET;
                        payload.camera.yaw = Camera::DEFAULT_YAW.to_radians();
                        payload.camera.pitch = Camera::DEFAULT_PITCH.to_radians();
                    }
                })
            })
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
                    payload.camera.position.x,
                    payload.camera.position.y,
                    payload.camera.position.z,
                    payload.camera.target.x,
                    payload.camera.target.y,
                    payload.camera.target.z,
                    payload.camera.yaw,
                    payload.camera.yaw.to_degrees(),
                    payload.camera.pitch,
                    payload.camera.pitch.to_degrees(),
                ));
                ui.label(format!("{:?}", payload.camera.mode))
            });
    }
}
