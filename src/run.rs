use tracing::{info, trace};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};

use crate::window::Window;

pub fn run(window: Window) {
    window.event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        // Event checking
        match event {
            // Check for app close event
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Closing game!");
                control_flow.set_exit();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => match key {
                    VirtualKeyCode::Escape => control_flow.set_exit(),
                    _ => trace!("Key pressed: {key:?}"),
                },
                // WindowEvent::CursorMoved { position, .. } => debug!("Cursor move: {position:?}"),
                _ => {}
            },
            Event::MainEventsCleared => {
                window.inner.request_redraw();
            }
            Event::RedrawRequested(_) => {}
            _ => {}
        }
    });
}