// TODO: Make crate from this module

use std::time::Instant;

use common::{
    block::{Block, BlockRepr},
    clock::ClockStats,
    coord::{ChunkId, GlobalCoord, CHUNK_CUBE},
};
use egui::{
    global_dark_light_mode_switch, ComboBox, Context, DragValue, FontDefinitions, Grid,
    RadioButton, Slider, Style, TopBottomPanel, Window,
};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::PresentMode;
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{
    render::{renderer::Renderer, RenderMode},
    scene::{
        camera::{Camera, CameraMode},
        chunk::ChunkManager,
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
    pub renderer: &'a mut Renderer,
}

/// Represents debug overlay state (windows, buttons, etc.)
pub struct DebugOverlayState {
    // UI Visibility
    /// Overlay top bar
    top_bar_visible: bool,
    /// Graphics tweaks window
    graphics_opened: bool,
    /// GPU timings
    gpu_stats_opened: bool,
    /// Camera tweaks window
    camera_opened: bool,
    /// Chunk tweaks window
    chunks_opened: bool,
    /// Block changer
    painter_opened: bool,

    // Sub states
    graphics_tweaks: GraphicsTweaks,
    painter: Painter,
}

impl DebugOverlayState {
    pub const fn new() -> Self {
        Self {
            top_bar_visible: true,
            graphics_opened: false,
            gpu_stats_opened: false,
            camera_opened: false,
            chunks_opened: false,
            painter_opened: false,
            graphics_tweaks: GraphicsTweaks::new(),
            painter: Painter::new(),
        }
    }

    // TODO: Shift+F3 shortcut to hide menu_bar
    pub fn draw(&mut self, ctx: &Context, payload: DebugPayload) {
        let DebugPayload {
            clock_stats,
            scene:
                Scene {
                    camera,
                    chunk_manager,
                    fps,
                    ..
                },
            renderer,
        } = payload;

        if self.top_bar_visible {
            TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    global_dark_light_mode_switch(ui);
                    ui.separator();
                    ui.menu_button("Game", |menu| {
                        if menu.button("GPU Stats").clicked() {
                            self.gpu_stats_opened = true;
                        }
                        if menu.button("Graphics").clicked() {
                            self.graphics_opened = true;
                        }
                    });
                    ui.menu_button("Scene", |menu| {
                        if menu.button("Camera").clicked() {
                            self.camera_opened = true;
                        }
                        if menu.button("ChunkManager").clicked() {
                            self.chunks_opened = true;
                        }
                        if menu.button("Reset Camera").clicked() {
                            camera.f_pos = Camera::DEFAULT_POSITION;
                            camera.f_rot = Camera::DEFAULT_ORIENTATION;
                            camera.set_mode(CameraMode::FirstPerson);
                        }
                    });
                    ui.menu_button("Cheats", |menu| {
                        if menu.button("Painter").clicked() {
                            self.painter_opened = true;
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

        Window::new("GPU Stats")
            .open(&mut self.gpu_stats_opened)
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
                ui.collapsing("Buffers", |ui| {
                    let (terrain_vertices, terrain_indices) = chunk_manager.terrain.values().fold(
                        (0, 0),
                        |(vertices, indices), chunk| {
                            (
                                vertices + chunk.vertex_buffer.length(),
                                indices + chunk.index_buffer.length(),
                            )
                        },
                    );
                    ui.label("Terrain Chunks:");
                    ui.label(format!("\tVertices: {}", terrain_vertices));
                    ui.label(format!("\tIndices: {}", terrain_indices));
                });
            });

        Window::new("Graphics")
            .open(&mut self.graphics_opened)
            .resizable(false)
            .show(ctx, |ui| {
                Grid::new("camera_tweaks")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Present Mode");
                        ComboBox::from_id_source("present_mode")
                            .selected_text(format!("{:?}", self.graphics_tweaks.present_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.graphics_tweaks.present_mode,
                                    PresentMode::Fifo,
                                    "Fifo",
                                );
                                ui.selectable_value(
                                    &mut self.graphics_tweaks.present_mode,
                                    PresentMode::Mailbox,
                                    "Mailbox",
                                );
                                ui.selectable_value(
                                    &mut self.graphics_tweaks.present_mode,
                                    PresentMode::Immediate,
                                    "Immediate",
                                );
                            });
                        ui.end_row();

                        ui.label("FPS Cap");
                        ui.add(
                            Slider::new(
                                &mut self.graphics_tweaks.fps,
                                Scene::FPS_MIN..=Scene::FPS_MAX,
                            )
                            .integer(),
                        );
                        ui.end_row();
                    });

                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        self.graphics_tweaks = GraphicsTweaks::new();
                    }
                    if ui.button("Apply").clicked() {
                        renderer.set_render_mode(self.graphics_tweaks.as_render_mode());
                        *fps = self.graphics_tweaks.fps;
                    }
                });
            });

        Window::new("Camera")
            .open(&mut self.camera_opened)
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
                                    camera.set_mode(CameraMode::FirstPerson);
                                }
                                if ui
                                    .add(RadioButton::new(
                                        matches!(camera.mode, CameraMode::ThirdPerson { .. }),
                                        "Third Person",
                                    ))
                                    .clicked()
                                {
                                    camera.set_mode(CameraMode::ThirdPerson);
                                }
                            });
                            ui.end_row();

                            ui.checkbox(&mut camera.smooth_position, "Smooth position");
                            ui.end_row();

                            ui.checkbox(&mut camera.smooth_rotation, "Smooth rotation");
                            ui.end_row();

                            ui.label("FOV");
                            ui.add(
                                Slider::new(&mut camera.f_fov, Camera::MIN_FOV..=Camera::MAX_FOV)
                                    .custom_formatter(|fov, _| {
                                        format!("{:.1}° ({:.2})", fov.to_degrees(), fov)
                                    }),
                            );
                            ui.end_row();

                            ui.label("Z Near");
                            ui.add(
                                Slider::new(
                                    &mut camera.near,
                                    Camera::MIN_Z_NEAR..=Camera::MAX_Z_NEAR,
                                )
                                .max_decimals(3),
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
                        Distance: {:.2}\n\
                        FOV: {:.3} {:.2}\n\
                        {:#?}",
                        camera.pos.x,
                        camera.pos.y,
                        camera.pos.z,
                        camera.rot.x,
                        camera.rot.x.to_degrees(),
                        camera.rot.y,
                        camera.rot.y.to_degrees(),
                        camera.dist,
                        camera.fov,
                        camera.fov.to_degrees(),
                        camera.mode
                    ));
                });
                ui.collapsing("Future Tracker", |ui| {
                    ui.label(format!(
                        "Position: x:{:.3} y:{:.3} z:{:.3}\n\
                        Yaw: {:.3} ({:.2})\n\
                        Pitch: {:.3} ({:.2})\n\
                        Distance: {:.2}\n\
                        FOV: {:.3} {:.2}\n",
                        camera.f_pos.x,
                        camera.f_pos.y,
                        camera.f_pos.z,
                        camera.f_rot.x,
                        camera.f_rot.x.to_degrees(),
                        camera.f_rot.y,
                        camera.f_rot.y.to_degrees(),
                        camera.f_dist,
                        camera.f_fov,
                        camera.f_fov.to_degrees(),
                    ));
                });
            });

        Window::new("ChunkManager")
            .open(&mut self.chunks_opened)
            .resizable(false)
            .show(ctx, |ui| {
                ui.collapsing("Tweaks", |ui| {
                    Grid::new("chunks_tweaks")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Draw distance");
                            ui.add(
                                DragValue::new(&mut chunk_manager.draw_distance)
                                    .fixed_decimals(0)
                                    .speed(1.0)
                                    .clamp_range(
                                        ChunkManager::MIN_DRAW_DISTANCE
                                            ..=ChunkManager::MAX_DRAW_DISTANCE,
                                    ),
                            );
                            ui.end_row();

                            if ui.button("Clear Mesh").clicked() {
                                chunk_manager.clear_mesh();
                            }
                            ui.end_row();

                            if ui.button("Cleanup").clicked() {
                                // TODO: Make GC tick
                                chunk_manager.cleanup();
                            }
                            ui.end_row();
                        });
                });

                ui.collapsing("Stats", |ui| {
                    Grid::new("chunk_manger_stats_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            let ChunkManager { logic, terrain, .. } = chunk_manager;

                            ui.label("Logic Chunks:");
                            ui.label(format!("{} ({})", logic.len(), logic.capacity()));
                            ui.end_row();

                            ui.label("Terrain Chunks:");
                            ui.label(format!("{} ({})", terrain.len(), terrain.capacity()));
                            ui.end_row();
                        });
                });
            });

        Window::new("Painter")
            .open(&mut self.painter_opened)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add(
                    DragValue::new(&mut self.painter.block)
                        .clamp_range(Block::MIN..=Block::MAX)
                        .custom_formatter(|id, _| {
                            format!("Selected Block: {:?}", Block::from(id as BlockRepr))
                        }),
                );

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Block Changer");

                            if ui.button("Set").clicked() {
                                if let Some(chunk) = chunk_manager
                                    .logic
                                    .get_mut(&self.painter.block_pos.to_chunk_id())
                                {
                                    chunk.blocks_mut()
                                        [self.painter.block_pos.to_block().flatten()] =
                                        Block::from(self.painter.block);
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.add(
                                DragValue::new(&mut self.painter.block_pos.x)
                                    .prefix("x: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.painter.block_pos.y)
                                    .prefix("y: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.painter.block_pos.z)
                                    .prefix("z: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                        });
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Chunk Filler");
                            if ui.button("Fill").clicked() {
                                if let Some(chunk) =
                                    chunk_manager.logic.get_mut(&self.painter.chunk_id)
                                {
                                    *chunk.blocks_mut() =
                                        [Block::from(self.painter.block); CHUNK_CUBE];
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                DragValue::new(&mut self.painter.chunk_id.x)
                                    .prefix("x: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.painter.chunk_id.y)
                                    .prefix("y: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.painter.chunk_id.z)
                                    .prefix("z: ")
                                    .fixed_decimals(0)
                                    .speed(1.0),
                            );
                        });
                    });
                });

                // TODO: Add button to set position to camera
                if ui.button("Reset").clicked() {
                    self.painter = Painter::new();
                }
            });
    }
}

pub struct GraphicsTweaks {
    fps: u32,
    present_mode: PresentMode,
}

impl GraphicsTweaks {
    pub const fn new() -> Self {
        Self {
            fps: Scene::FPS_DEFAULT,
            present_mode: RenderMode::new().present_mode,
        }
    }

    pub fn as_render_mode(&self) -> RenderMode {
        RenderMode {
            present_mode: self.present_mode,
        }
    }
}

pub struct Painter {
    block_pos: GlobalCoord,
    chunk_id: ChunkId,
    block: BlockRepr,
}

impl Painter {
    pub const fn new() -> Self {
        Self {
            block_pos: GlobalCoord::ZERO,
            chunk_id: ChunkId::ZERO,
            block: Block::Stone as BlockRepr,
        }
    }
}
