use tokio::runtime::Runtime;
use tracing::{error, warn};
use winit::{
    dpi::LogicalSize,
    event::ModifiersState,
    window::{CursorGrabMode, Window as WinitWindow, WindowBuilder},
};

use crate::{
    consts::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH},
    render::{error::RenderError, renderer::Renderer},
    types::EventLoop,
    utils::VERSION,
};

use event::Event;

pub mod event;

/// Handler for Winit Window and EventLoop
pub struct Window {
    /// winit window handle
    inner: WinitWindow,

    renderer: Renderer,

    pub fullscreen: bool,
    cursor_grabbed: bool,

    events: Vec<Event>,
    modifiers: ModifiersState,

    // Deduplicated events
    resized: bool,
    toggle_fullscreen: bool,
}

impl Window {
    pub const INITIAL_WIDTH: u32 = 1280;
    pub const INITIAL_HEIGHT: u32 = 720;

    pub fn new(runtime: &Runtime) -> Result<(Self, EventLoop), RenderError> {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_resizable(true)
            .with_transparent(false)
            .with_maximized(true)
            .with_min_inner_size(LogicalSize::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT))
            .with_title(format!("ECG v{VERSION}"))
            .with_inner_size(LogicalSize::new(Self::INITIAL_WIDTH, Self::INITIAL_HEIGHT))
            .build(&event_loop)
            .unwrap();

        let renderer = Renderer::new(&window, runtime)?;

        Ok((
            Self {
                inner: window,
                renderer,
                cursor_grabbed: false,
                fullscreen: false,
                events: Vec::new(),
                modifiers: Default::default(),
                resized: false,
                toggle_fullscreen: false,
            },
            event_loop,
        ))
    }

    pub fn inner(&self) -> &WinitWindow {
        &self.inner
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    /// Grab cursor and make it invisible
    pub fn grab_cursor(&mut self, grab: bool) {
        self.cursor_grabbed = grab;

        if grab {
            self.inner
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| {
                    warn!("Failed to grab cursor. Retrying in 'Locked' mode");
                    self.inner.set_cursor_grab(CursorGrabMode::Locked)
                })
                .unwrap_or_else(|_| error!("Failed to grab cursor in both modes"));
        } else {
            self.inner
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap_or_else(|_| error!("Failed to release cursor"));
        }

        self.inner.set_cursor_visible(!grab);
    }
}
