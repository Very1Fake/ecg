use anyhow::Result;
use tracing::{error, info};
use wgpu::SurfaceError;
use winit::event::{Event, WindowEvent};

use crate::{game::Game, types::EventLoop, utils::ExitCode, window::Window};

pub async fn run(event_loop: EventLoop, mut window: Window, mut game: Game) -> Result<()> {
    game.pause(false, &mut window)?;

    event_loop.run(move |event, _, control_flow| {
        // Continuos rendering
        control_flow.set_poll();

        // Event checking
        match event {
            // Check for app close event
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Closing game!");
                control_flow.set_exit_with_code(ExitCode::Ok.as_int());
            }
            Event::WindowEvent { .. } | Event::DeviceEvent { .. } => {
                game.input(event, control_flow, &mut window)
            }
            Event::MainEventsCleared => {
                window.inner.request_redraw();
            }
            Event::RedrawRequested(id) if id == window.inner.id() => {
                // Update state
                game.update();

                // Render game frame
                match game.render(&window) {
                    Ok(_) => {}
                    Err(err) => {
                        match err.downcast::<SurfaceError>() {
                            // If surface lost, try to recover it by reconfiguring
                            Ok(SurfaceError::Lost) => game.graphics.recover_surface(),
                            Ok(SurfaceError::OutOfMemory) => {
                                error!("GPU ran out of memory. Exiting");
                                control_flow
                                    .set_exit_with_code(ExitCode::OutOfVideoMemory.as_int());
                            }
                            Ok(_) => {}
                            Err(err) => error!("{err:?}"),
                        }
                    }
                }
            }
            _ => {}
        }
    });
}
