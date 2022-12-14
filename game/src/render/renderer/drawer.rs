use std::iter::once;

use wgpu::{
    Color, CommandEncoder, Device, IndexFormat, LoadOp, Operations, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    SurfaceTexture, TextureView, TextureViewDescriptor,
};
use wgpu_profiler::scope::{ManualOwningScope, OwningScope, Scope};

use crate::render::buffer::{Buffer, DynamicBuffer};
use crate::render::pipelines::GlobalsBindGroup;

use crate::render::primitives::instance::RawInstance;
use crate::render::{model::Model, primitives::vertex::Vertex, texture::Texture};
use crate::scene::chunk::TerrainChunk;

use super::pipelines::Pipelines;
use super::Renderer;

#[cfg(feature = "debug_overlay")]
use {
    egui::FullOutput,
    egui_wgpu_backend::{BackendError, ScreenDescriptor},
    egui_winit_platform::Platform,
    wgpu::SurfaceConfiguration,
};

struct RendererBorrow<'frame> {
    device: &'frame Device,
    queue: &'frame Queue,
    pipelines: &'frame Pipelines,
    depth_texture: &'frame Texture,
    #[cfg(feature = "debug_overlay")]
    surface_config: &'frame SurfaceConfiguration,
    #[cfg(feature = "debug_overlay")]
    egui_render_pass: &'frame mut egui_wgpu_backend::RenderPass,
}

/// Used to draw on current frame.
///
/// Draw calls will be submitted when the object is dropped.
pub struct Drawer<'frame> {
    encoder: Option<ManualOwningScope<'frame, CommandEncoder>>,
    renderer: RendererBorrow<'frame>,
    output_texture: Option<SurfaceTexture>,
    output_view: TextureView,
    globals: &'frame GlobalsBindGroup,
}

impl<'frame> Drawer<'frame> {
    pub fn new(
        encoder: CommandEncoder,
        renderer: &'frame mut Renderer,
        output_texture: SurfaceTexture,
        globals: &'frame GlobalsBindGroup,
    ) -> Self {
        let output_view = output_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let encoder =
            ManualOwningScope::start("frame", &mut renderer.profiler, encoder, &renderer.device);

        Self {
            encoder: Some(encoder),
            renderer: RendererBorrow {
                device: &renderer.device,
                queue: &renderer.queue,
                pipelines: &renderer.pipelines,
                depth_texture: &renderer.depth_texture,
                #[cfg(feature = "debug_overlay")]
                surface_config: &renderer.config,
                #[cfg(feature = "debug_overlay")]
                egui_render_pass: &mut renderer.egui_render_pass,
            },
            output_texture: Some(output_texture),
            output_view,
            globals,
        }
    }

    /// Returns sub drawer for the first pass
    pub fn first_pass(&mut self) -> FirstPassDrawer {
        let mut render_pass = self.encoder.as_mut().unwrap().scoped_render_pass(
            "first_pass",
            self.renderer.device,
            &RenderPassDescriptor {
                label: Some("FirstPass"),
                // Where to we draw colors
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.output_view,
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            },
        );

        render_pass.set_bind_group(0, &self.globals.inner, &[]);

        FirstPassDrawer {
            render_pass,
            renderer: &self.renderer,
            pipelines: self.renderer.pipelines,
        }
    }

    // FIX: Handle egui textures better
    /// Draw debug overlay
    #[cfg(feature = "debug_overlay")]
    pub fn draw_overlay(
        &mut self,
        platform: &mut Platform,
        scale_factor: f32,
    ) -> Result<(), BackendError> {
        common_log::span!(_guard, "DrawOverlay", "Draw::Overlay");
        // Finalize frame
        // FIX: Fixes cursor flickering, but cursor icons won't change

        let FullOutput {
            textures_delta,
            shapes,
            ..
        } = platform.end_frame(None);

        // Tesselate shapes
        let paint_jobs = platform.context().tessellate(shapes);

        let screen_descriptor = &ScreenDescriptor {
            physical_width: self.renderer.surface_config.width,
            physical_height: self.renderer.surface_config.height,
            scale_factor,
        };

        // Send textures and update buffers
        self.renderer.egui_render_pass.add_textures(
            self.renderer.device,
            self.renderer.queue,
            &textures_delta,
        )?;
        self.renderer.egui_render_pass.update_buffers(
            self.renderer.device,
            self.renderer.queue,
            &paint_jobs,
            screen_descriptor,
        );

        // Record all commands to encoder
        self.renderer.egui_render_pass.execute(
            self.encoder.as_mut().unwrap(),
            &self.output_view,
            &paint_jobs,
            screen_descriptor,
            None,
        )?;

        // Cleanup egui textures
        self.renderer
            .egui_render_pass
            .remove_textures(textures_delta)?;

        Ok(())
    }
}

impl<'frame> Drop for Drawer<'frame> {
    fn drop(&mut self) {
        let encoder = self.encoder.take().unwrap();

        let (mut encoder, profiler) = encoder.end_scope();
        profiler.resolve_queries(&mut encoder);

        // Submit render operations
        self.renderer.queue.submit(once(encoder.finish()));

        // Show rendered frame
        self.output_texture.take().unwrap().present();

        profiler.end_frame().expect("GPU Profiler error!");
    }
}

// TODO: Add render texture to renderer and use it here (for upscale/downscale)
/// Sub drawer that handles first render pass (terrain, figures)
#[must_use]
pub struct FirstPassDrawer<'pass> {
    render_pass: OwningScope<'pass, RenderPass<'pass>>,
    renderer: &'pass RendererBorrow<'pass>,
    pipelines: &'pass Pipelines,
}

impl<'pass> FirstPassDrawer<'pass> {
    /// Draw debug pyramid
    pub fn draw_pyramid(&mut self, vertices: &'pass Buffer<Vertex>, indices: &'pass Buffer<u16>) {
        let mut render_pass = self.render_pass.scope("pyramid", self.renderer.device);

        render_pass.set_pipeline(&self.pipelines.terrain.inner);
        render_pass.set_vertex_buffer(0, vertices.buffer.slice(..));
        render_pass.set_index_buffer(indices.buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..Vertex::INDICES.len() as u32, 0, 0..1);
    }

    /// Returns TerrainDrawer
    pub fn terrain_drawer(&mut self) -> TerrainDrawer<'_, 'pass> {
        let mut render_pass = self.render_pass.scope("terrain", self.renderer.device);

        render_pass.set_pipeline(&self.pipelines.terrain.inner);

        TerrainDrawer { render_pass }
    }

    // FIX: Make `FiguresDrawer` sub drawer for this operation
    pub fn draw_figure<T: Model>(
        &mut self,
        model: &'pass T,
        instances: &'pass DynamicBuffer<RawInstance>,
    ) {
        let mut render_pass = self.render_pass.scope("figure", self.renderer.device);

        let (index_buffer, count) = model.get_indices();

        render_pass.set_pipeline(&self.pipelines.figure.inner);
        render_pass.set_vertex_buffer(0, model.get_vertices().slice(..));
        render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
        // TODO: Make safe cast
        render_pass.draw_indexed(0..count, 0, 0..instances.length() as u32);
    }
}

#[must_use]
pub struct TerrainDrawer<'pass_ref, 'pass: 'pass_ref> {
    render_pass: Scope<'pass_ref, RenderPass<'pass>>,
}

impl<'pass_ref, 'pass: 'pass_ref> TerrainDrawer<'pass_ref, 'pass> {
    /// Draw terrain chunk
    pub fn draw(&mut self, chunk: &'pass TerrainChunk) {
        self.render_pass
            .set_vertex_buffer(0, chunk.vertex_buffer.buffer.slice(..));
        self.render_pass
            .set_index_buffer(chunk.index_buffer.buffer.slice(..), IndexFormat::Uint32);
        self.render_pass
            .draw_indexed(0..chunk.index_buffer.length() as u32, 0, 0..1);
    }
}
