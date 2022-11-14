use common::clock::Clock;
use tokio::runtime::Runtime;
use tracing::{debug, debug_span, info};
use winit::{event::WindowEvent, event_loop::ControlFlow};

pub mod bootstrap;
pub mod consts;
#[cfg(feature = "debug_overlay")]
pub mod egui;
pub mod error;
pub mod render;
pub mod scene;
pub mod types;
pub mod utils;
pub mod window;

#[cfg(feature = "debug_overlay")]
use crate::egui::DebugOverlay;

use crate::{
    scene::Scene,
    types::{EventLoop, WEvent},
    utils::ExitCode,
    window::Window,
};

/// Game instance
pub struct Game {
    pub window: Window,
    pub runtime: Runtime,
    pub clock: Clock,

    // Debug UI
    #[cfg(feature = "debug_overlay")]
    pub debug_overlay: DebugOverlay,
}

impl Game {
    pub const TARGET_FPS: u32 = 60;
    pub const BACKGROUND_FPS: u32 = 30;

    pub fn new(window: Window, runtime: Runtime) -> Self {
        // Logging span
        let _span = debug_span!("game_init").entered();

        info!("Creating new game instance");

        #[cfg(feature = "debug_overlay")]
        let debug_overlay = {
            info!("Initializing debug UI");
            DebugOverlay::new(window.inner())
        };

        Self {
            window,
            runtime,
            clock: Clock::new(Clock::tps_to_duration(Self::TARGET_FPS)),
            #[cfg(feature = "debug_overlay")]
            debug_overlay,
        }
    }

    pub fn tick(&mut self, control_flow: &mut ControlFlow, scene: &mut Scene) {
        // Fetch occurred events
        let events = self.window.fetch();

        // Update game state
        let exit = scene.update(self, events, self.clock.duration());

        if exit {
            *control_flow = ControlFlow::Exit;
        }

        // Render
        {
            #[cfg(feature = "debug_overlay")]
            let scale_factor = self.window.inner().scale_factor() as f32;

            if let Some(mut drawer) = self
                .window
                .renderer_mut()
                .start_frame(&scene.globals_bind_group)
                .expect("Unrecoverable render error when starting a new frame")
            {
                scene.draw(drawer.first_pass());

                #[cfg(feature = "debug_overlay")]
                if scene.show_overlay {
                    drawer
                        .draw_debug_overlay(&mut self.debug_overlay.platform, scale_factor)
                        .expect("Unrecoverable render error when drawing debug overlay");
                }
            }
        }

        // Wait for next frame
        if !exit {
            // Lower target frame time when the game window is not focused
            self.clock.target = Clock::tps_to_duration(if self.window.focused {
                Self::TARGET_FPS
            } else {
                Self::BACKGROUND_FPS
            });
            self.clock.tick();
        }
    }

    pub fn run(mut self, event_loop: EventLoop) {
        // TODO: PlayStates
        debug!("Initializing game scene");
        let mut scene = Scene::new(&mut self.window);

        debug!("Entering game loop");
        event_loop.run(move |event, _, control_flow| {
            // Continuos rendering
            control_flow.set_poll();

            #[cfg(feature = "debug_overlay")]
            {
                // Let debug UI handle occurred event, if cursor detached from camera
                if scene.show_overlay
                    && self
                        .debug_overlay
                        .handle_event(&event, self.window.cursor_grabbed())
                {
                    return;
                }
            }

            // Event checking
            match event {
                // Check for app close event
                WEvent::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Closing game!");
                    control_flow.set_exit_with_code(ExitCode::Ok.as_int());
                }
                WEvent::WindowEvent { event, .. } => self.window.handle_window_event(event),
                WEvent::DeviceEvent { event, .. } => self.window.handle_device_event(event),
                WEvent::MainEventsCleared => self.tick(control_flow, &mut scene),
                _ => {}
            }
        });
    }
}
