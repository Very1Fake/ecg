use tracing::{error, info};
use wgpu::SurfaceError;
use winit::event::{Event, WindowEvent};

use crate::{game::Game, utils::ExitCode, window::Window};

pub async fn run(window: Window, mut game: Game) {
    window.event_loop.run(move |event, _, control_flow| {
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
                game.input(event, control_flow)
            }
            Event::MainEventsCleared => {
                window.inner.request_redraw();
            }
            Event::RedrawRequested(id) if id == window.inner.id() => {
                game.update();
                match game.render() {
                    Ok(_) => {}
                    // If surface lost, try to recover it by reconfiguring
                    Err(SurfaceError::Lost) => game.graphics.recover_surface(),
                    Err(SurfaceError::OutOfMemory) => {
                        error!("GPU ran out of memory. Exiting");
                        control_flow.set_exit_with_code(ExitCode::OutOfVideoMemory.as_int());
                    }
                    Err(err) => error!("{err:?}"),
                }
            }
            _ => {}
        }
    });
}
