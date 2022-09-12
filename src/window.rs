use anyhow::Result;
use winit::{event_loop::EventLoop, window::{WindowBuilder, Window as WinitWindow}};

/// Handler for Winit Window and EventLoop
pub struct Window {
    pub inner: WinitWindow,
    pub event_loop: EventLoop<()>,
}

impl Window {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoop::new();

        Ok(Self {
            inner: WindowBuilder::new().build(&event_loop)?,
            event_loop,
        })
    }
}
