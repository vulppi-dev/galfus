mod pipeline;
mod textures;

use egui::ClippedPrimitive;

use crate::core::ui::renderer::pipeline::UiPipeline;
use crate::core::ui::renderer::textures::UiTextureStore;

pub struct UiRenderer {
    pipeline: UiPipeline,
    textures: UiTextureStore,
}

impl UiRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, target_format: wgpu::TextureFormat) -> Self {
        let pipeline = UiPipeline::new(device, target_format);
        let textures = UiTextureStore::new(device, queue, pipeline.bind_group_layout(), pipeline.sampler(), pipeline.uniform_buffer());
        Self { pipeline, textures }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: &egui::TexturesDelta,
    ) {
        self.textures
            .update_textures(device, queue, delta, self.pipeline.bind_group_layout(), self.pipeline.sampler(), self.pipeline.uniform_buffer());
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        target_format: wgpu::TextureFormat,
        target_size: glam::UVec2,
        pixels_per_point: f32,
        clipped_primitives: &[ClippedPrimitive],
    ) {
        self.pipeline.ensure_target_format(device, target_format);
        self.pipeline.update_uniforms(queue, target_size);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(self.pipeline.pipeline());

        for ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
            let egui::epaint::Primitive::Mesh(mesh) = primitive else {
                continue;
            };

            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }

            let clip_min_x = (clip_rect.min.x * pixels_per_point).max(0.0);
            let clip_min_y = (clip_rect.min.y * pixels_per_point).max(0.0);
            let clip_max_x = (clip_rect.max.x * pixels_per_point).min(target_size.x as f32);
            let clip_max_y = (clip_rect.max.y * pixels_per_point).min(target_size.y as f32);

            let scissor_width = (clip_max_x - clip_min_x).max(0.0);
            let scissor_height = (clip_max_y - clip_min_y).max(0.0);
            if scissor_width <= 0.0 || scissor_height <= 0.0 {
                continue;
            }

            render_pass.set_scissor_rect(
                clip_min_x.round() as u32,
                clip_min_y.round() as u32,
                scissor_width.round() as u32,
                scissor_height.round() as u32,
            );

            let (vertex_buffer, index_buffer, index_count) =
                self.pipeline.build_mesh_buffers(device, mesh);

            let texture = self.textures.get(mesh.texture_id);

            render_pass.set_bind_group(0, texture.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
}
