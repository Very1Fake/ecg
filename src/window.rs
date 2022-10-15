use anyhow::{Context, Result};
use tracing::{debug, error, warn};
use winit::{
    dpi::PhysicalSize,
    window::{CursorGrabMode, Window as WinitWindow, WindowBuilder},
};

use crate::{types::EventLoop, utils::VERSION};

/// Handler for Winit Window and EventLoop
pub struct Window {
    pub inner: WinitWindow,
    pub cursor_grabbed: bool,
}

impl Window {
    pub const INITIAL_WIDTH: u32 = 1280;
    pub const INITIAL_HEIGHT: u32 = 720;

    pub fn new() -> Result<(Self, EventLoop)> {
        let event_loop = EventLoop::new();

        Ok((
            Self {
                inner: WindowBuilder::new()
                    .with_decorations(true)
                    .with_resizable(true)
                    .with_transparent(false)
                    .with_title(format!("ECG v{VERSION}"))
                    .with_inner_size(PhysicalSize::new(Self::INITIAL_WIDTH, Self::INITIAL_HEIGHT))
                    .build(&event_loop)?,
                cursor_grabbed: false,
            },
            event_loop,
        ))
    }

    /// Apply cursor grab state
    #[inline]
    pub fn apply_cursor_visibility(&self) {
        self.inner.set_cursor_visible(!self.cursor_grabbed);
    }

    /// Grab cursor and make it invisible
    pub fn grab_cursor(&mut self, grab: bool) -> Result<()> {
        self.cursor_grabbed = grab;

        if grab {
            debug!("Grabbing cursor in 'Confined' mode");
            self.inner
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| {
                    warn!("Failed to grab cursor. Retrying in 'Locked' mode");
                    self.inner.set_cursor_grab(CursorGrabMode::Locked)
                })
                .with_context(|| {
                    error!("Failed to grab cursor in both modes");
                    "While grabbing cursor"
                })?;
        } else {
            debug!("Releasing cursor");
            self.inner.set_cursor_grab(CursorGrabMode::None)?;
        }

        self.apply_cursor_visibility();

        Ok(())
    }
}
