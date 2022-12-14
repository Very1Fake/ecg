use std::time::Duration;

use common::{
    block::Block,
    coord::{ChunkId, CHUNK_SQUARE},
};
use common_log::span;
use wgpu::BufferUsages;
use winit::event::{ElementState, VirtualKeyCode};

use crate::{
    render::{
        buffer::{Buffer, DynamicBuffer},
        pipelines::{GlobalModel, Globals, GlobalsBindGroup},
        primitives::{
            instance::{Instance, RawInstance},
            vertex::Vertex,
        },
        renderer::drawer::FirstPassDrawer,
    },
    scene::chunk::LogicChunk,
    types::{F32x3, Rotation},
    window::{
        event::{Event, Input},
        Window,
    },
    Game,
};

use self::{
    camera::{Camera, CameraController, CameraMode},
    chunk::ChunkManager,
    figure::voxel::Voxel,
};

pub mod camera;
pub mod chunk;
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

    // World
    pub chunk_manager: ChunkManager,

    // Objects
    pub pyramid_vertices: Buffer<Vertex>,
    pub pyramid_indices: Buffer<u16>,
    pub voxel: Voxel,
    pub voxel_instance: Instance,
    pub voxel_instance_buffer: DynamicBuffer<RawInstance>,

    // TODO: Store in settings
    pub fps: u32,

    // UI
    force_cursor_grub: bool,

    #[cfg(feature = "debug_overlay")]
    pub show_overlay: bool,
}

impl Scene {
    pub const FPS_MIN: u32 = 10;
    pub const FPS_DEFAULT: u32 = 60;
    pub const FPS_MAX: u32 = 360;

    /// Create new `Scene`
    pub fn new(window: &mut Window) -> Self {
        span!(_guard, "new", "Scene::new");
        window.grab_cursor(true);
        let renderer = window.renderer_mut();

        let resolution = renderer.resolution();

        let model = GlobalModel {
            globals: renderer.create_consts(&[Globals::default()]),
        };

        let globals_bind_group = renderer.bind_globals(&model);

        let voxel_instance = Instance::new(F32x3::ZERO, Rotation::IDENTITY);
        let voxel_instance_buffer = DynamicBuffer::new(&renderer.device, 1, BufferUsages::VERTEX);
        voxel_instance_buffer.update(&renderer.queue, &[voxel_instance.as_raw()], 0);

        let mut chunk_manager = ChunkManager::new();

        chunk_manager.logic.insert(ChunkId::ZERO, {
            let mut chunk = LogicChunk::new();
            chunk
                .blocks_mut()
                .iter_mut()
                .skip(CHUNK_SQUARE * 8)
                .zip(Block::ALL.iter())
                .for_each(|(block, block_type)| *block = *block_type);
            chunk
        });

        Self {
            model,
            globals_bind_group,

            camera: Camera::new(
                resolution.x as f32 / resolution.y as f32,
                CameraMode::FirstPerson,
            ),
            camera_controller: CameraController::default(),

            chunk_manager,

            pyramid_vertices: Buffer::new(&renderer.device, Vertex::PYRAMID, BufferUsages::VERTEX),
            pyramid_indices: Buffer::new(&renderer.device, Vertex::INDICES, BufferUsages::INDEX),

            voxel: Voxel::new(&renderer.device),
            voxel_instance,
            voxel_instance_buffer,

            fps: Scene::FPS_DEFAULT,

            force_cursor_grub: true,

            #[cfg(feature = "debug_overlay")]
            show_overlay: false,
        }
    }

    fn toggle_cursor_grub(&mut self) {
        self.force_cursor_grub = !self.force_cursor_grub;
        self.camera_controller.reset();
    }

    // FIX: Make `Settings` to pass overlay toggles
    /// Update scene state. Return `false` if should close the game
    pub fn tick(&mut self, game: &mut Game, events: Vec<Event>, tick_dur: Duration) -> bool {
        span!(_guard, "tick", "Scene::tick");

        let mut exit = false;

        // Handle events
        events.into_iter().for_each(|event| match event {
            Event::Close => exit = true,
            Event::Resize(size) => self.camera.aspect = size.x as f32 / size.y as f32,
            // FIX: Abnormal touchpad sensitivity
            Event::MouseMove(delta, true) => self.camera.rotate(delta),
            Event::Zoom(delta, true) => self.camera.zoom(delta),
            Event::Input(Input::Key(key), state, modifiers) => {
                match key {
                    VirtualKeyCode::Escape => exit = true,
                    VirtualKeyCode::P if matches!(state, ElementState::Released) => {
                        self.toggle_cursor_grub()
                    }
                    #[cfg(feature = "debug_overlay")]
                    VirtualKeyCode::F3
                        if matches!(state, ElementState::Released) && modifiers.shift() =>
                    {
                        game.overlay.toggle_top_bar();
                    }
                    #[cfg(feature = "debug_overlay")]
                    VirtualKeyCode::F3 if matches!(state, ElementState::Released) => {
                        self.show_overlay = !self.show_overlay
                    }
                    _ => {}
                }

                if self.force_cursor_grub {
                    self.camera_controller.virtual_key(key, state);
                }
            }
            Event::Focused(focused) => self.force_cursor_grub = focused,
            _ => {}
        });

        // Update debug overlay
        #[cfg(feature = "debug_overlay")]
        game.overlay.update(crate::egui::DebugPayload {
            clock_stats: game.clock.stats(),
            scene: self,
            renderer: game.window.renderer_mut(),
        });

        // Update camera
        self.camera.update(tick_dur);
        self.camera_controller
            .move_camera(&mut self.camera, tick_dur);
        game.window.renderer().update_consts(
            &self.model.globals,
            &[Globals::new(self.camera.proj_mat(), self.camera.view_mat())],
        );

        self.chunk_manager
            .maintain(&game.window.renderer().device, &game.runtime, &self.camera);

        // Update voxel position
        if matches!(self.camera.mode, CameraMode::ThirdPerson) {
            self.voxel_instance.position = self.camera.pos;
            game.window.renderer().update_dynamic_buffer(
                &self.voxel_instance_buffer,
                &[self.voxel_instance.as_raw()],
            );
        }

        game.window.grab_cursor(self.force_cursor_grub);

        exit
    }

    /// Draw in-game objects
    pub fn draw<'a>(&'a self, mut drawer: FirstPassDrawer<'a>) {
        span!(_guard, "draw", "Scene::draw");

        // Draw "terrain"
        {
            // Test pyramid
            drawer.draw_pyramid(&self.pyramid_vertices, &self.pyramid_indices);

            let mut drawer = drawer.terrain_drawer();

            self.chunk_manager
                .terrain
                .values()
                .for_each(|chunk| drawer.draw(chunk));
        }

        // Draw figures
        drawer.draw_figure(&self.voxel, &self.voxel_instance_buffer);
    }
}
