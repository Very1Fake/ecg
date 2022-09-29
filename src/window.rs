use anyhow::{Context, Result};
use tracing::{debug, error, warn};
use winit::window::{CursorGrabMode, Window as WinitWindow, WindowBuilder};

use crate::types::EventLoop;

/// Handler for Winit Window and EventLoop
pub struct Window {
    pub inner: WinitWindow,
}

impl Window {
    pub fn new() -> Result<(Self, EventLoop)> {
        let event_loop = EventLoop::new();

        Ok((
            Self {
                inner: WindowBuilder::new().build(&event_loop)?,
            },
            event_loop,
        ))
    }

    /// Grab cursor and make it invisible
    pub fn grab_cursor(&self, grab: bool) -> Result<()> {
        if grab {
            debug!("Grabbing cursor in 'Confined' mode");

            self.inner.set_cursor_visible(false);
            self.inner
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| {
                    warn!("Failed to grab cursor. Retrying in 'Locked' mode");
                    self.inner.set_cursor_grab(CursorGrabMode::Locked)
                })
                .with_context(|| {
                    error!("Failed to grab cursor in both modes");
                    "While grabbing cursor"
                })
                .unwrap();
        } else {
            debug!("Releasing cursor");

            self.inner.set_cursor_visible(true);
            self.inner.set_cursor_grab(CursorGrabMode::None).unwrap();
        }

        Ok(())
    }
}
