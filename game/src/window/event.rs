use std::mem::replace;

use common_log::span;
use tracing::debug;
use winit::{
    dpi::PhysicalSize,
    event::{
        DeviceEvent, ElementState, ModifiersState, MouseButton, MouseScrollDelta, ScanCode,
        VirtualKeyCode, WindowEvent,
    },
    window::Fullscreen,
};

use crate::types::{F32x2, U32x2};

use super::Window;

/// Represents input from keyboard and mouse
#[derive(Clone, Copy, Debug)]
pub enum Input {
    Key(VirtualKeyCode),
    Mouse(MouseButton),
    ScanCode(ScanCode),
}

/// Represents incoming events
#[derive(Clone, Debug)]
pub enum Event {
    /// The window close request
    Close,
    /// The window has been resized
    Resize(U32x2),
    /// The cursor has been moved across the window
    MouseMove(F32x2, bool),
    // A mouse button has been pressed/released
    // TODO: Use this for mouse input after adding GameInputs
    // MouseButton(MouseButton, ElementState),
    /// A mouse wheel has been scrolled
    Zoom(f32, bool),
    // TODO: Add GameInput and keybindings
    /// A keyboard button has been pressed/released
    Input(Input, ElementState, ModifiersState),
    /// The window is (un)focused
    Focused(bool),
}

/// Window logic for processing incoming events
impl Window {
    // TODO: Don't hardcode this
    const MOTION_SENSITIVITY: f32 = 2.5;
    const EVENTS_PREALLOCATE: usize = 4;

    pub fn handle_window_event(&mut self, event: WindowEvent) {
        // TODO: Check out occluded event
        match event {
            WindowEvent::Resized(_) => self.resized = true,
            WindowEvent::CloseRequested => self.events.push(Event::Close),
            WindowEvent::Focused(focused) => {
                self.focused = focused;
                self.events.push(Event::Focused(focused))
            }
            WindowEvent::KeyboardInput {
                input,
                is_synthetic,
                ..
            } => {
                match input.virtual_keycode {
                    // Ignore synthetic Tab presses from alt-tabbing
                    Some(VirtualKeyCode::Tab) if is_synthetic => {}
                    // Ignore synthetic Alt-F4
                    Some(VirtualKeyCode::F4) if self.modifiers.alt() => {}
                    Some(VirtualKeyCode::F11) if matches!(input.state, ElementState::Released) => {
                        self.toggle_fullscreen = true
                    }
                    virtual_keycode => self.events.push(Event::Input(
                        match virtual_keycode {
                            Some(key) => Input::Key(key),
                            None => Input::ScanCode(input.scancode),
                        },
                        input.state,
                        self.modifiers,
                    )),
                };
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers,
            WindowEvent::MouseWheel { delta, .. } => self.events.push(Event::Zoom(
                {
                    -(match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pixel) => (pixel.y * 16.0) as f32,
                    })
                },
                self.cursor_grabbed,
            )),
            WindowEvent::MouseInput { state, button, .. } => {
                self.events
                    .push(Event::Input(Input::Mouse(button), state, self.modifiers))
            }
            // TODO: Throw event when UI is implemented
            WindowEvent::ScaleFactorChanged { .. } => self.resized = true,
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        #[cfg(target_os = "linux")]
        const MOTION_FIX: f32 = 0.1;
        #[cfg(not(target_os = "linux"))]
        const MOTION_FIX: f32 = 1.0;

        if let DeviceEvent::MouseMotion { delta } = event {
            self.events.push(Event::MouseMove(
                F32x2::new(delta.0 as f32, delta.1 as f32) * Self::MOTION_SENSITIVITY * MOTION_FIX,
                self.cursor_grabbed,
            ))
        }
    }

    pub fn fetch_events(&mut self) -> Vec<Event> {
        span!(_guard, "fetch_events", "Window::fetch_event");

        // Handle deduplicated resize event
        if self.resized {
            self.resized = false;
            let size = {
                let PhysicalSize { width, height } = self.inner.inner_size();
                U32x2::new(width, height)
            };

            self.renderer.on_resize(size);

            // Emit event to notify UI and scene
            self.events.push(Event::Resize(size));
        }

        // Handle deduplicated fullscreen toggle event
        if self.toggle_fullscreen {
            self.toggle_fullscreen = false;

            match self.inner.fullscreen() {
                Some(_) => {
                    debug!("Switching to windowed mode");
                    self.inner.set_fullscreen(None)
                }
                None => {
                    // Available fullscreen modes for primary monitor
                    let mut modes = self
                        .inner
                        .primary_monitor()
                        .expect("Can't identify primary monitor")
                        .video_modes()
                        .collect::<Vec<_>>();

                    // Sort modes by size
                    modes.sort_by_cached_key(|mode| {
                        let size = mode.size();
                        size.height * size.width
                    });

                    let mode = modes.last().expect("Proper fullscreen mode not found");

                    debug!(
                        size = ?mode.size(),
                        bit_depth = mode.bit_depth(),
                        refresh_rate_millihertz = mode.refresh_rate_millihertz(),
                        "Switching to exclusive fullscreen mode"
                    );
                    self.inner
                        .set_fullscreen(Some(Fullscreen::Exclusive(mode.clone())));
                }
            }
        }

        replace(
            &mut self.events,
            Vec::with_capacity(Self::EVENTS_PREALLOCATE),
        )
    }
}
