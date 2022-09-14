use std::iter::once;

use tracing::debug;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, SurfaceError, TextureViewDescriptor,
};
use winit::{
    event::{KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::graphics::Graphics;

/// Game instance
pub struct Game {
    pub graphics: Graphics,
}

impl Game {
    pub fn new(graphics: Graphics) -> Self {
        Self { graphics }
    }

    pub fn input(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match key {
                VirtualKeyCode::Escape => control_flow.set_exit(),
                _ => debug!("Key pressed: {key:?}"),
            },
            WindowEvent::Resized(size) => self.graphics.resize(size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.graphics.resize(*new_inner_size)
            }
            // WindowEvent::CursorMoved { position, .. } => debug!("Cursor move: {position:?}"),
            _ => {}
        }
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        // Texture for drawing (frame)
        let output = self.graphics.surface.get_current_texture()?;

        // View for texture, required to control how the rendering code interacts with the texture
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Used to send series of operations to GPU
        let mut encoder = self
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Block Pipe Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Block Render Pass"),
            // Where to we draw colors
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    // Where to pick the previous frame.
                    // Clears screen with specified color
                    // NOTE: Right now used as simple skybox
                    load: LoadOp::Clear(Color {
                        r: 0.458,
                        g: 0.909,
                        b: 1.0,
                        a: 1.0,
                    }),
                    // Write results to texture
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.graphics.render_pipeline);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        // Submit render operations
        self.graphics.queue.submit(once(encoder.finish()));
        // Show rendered frame
        output.present();

        Ok(())
    }
}
