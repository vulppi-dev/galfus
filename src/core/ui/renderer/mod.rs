mod pipeline;
mod textures;

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use egui::ClippedPrimitive;

use crate::core::ui::renderer::pipeline::{UiPipeline, UiVertex};
use crate::core::ui::renderer::textures::UiTextureStore;

pub struct ExternalTextureInput {
    pub id: u64,
    pub view: wgpu::TextureView,
    pub size: [u32; 2],
    pub source_ptr: usize,
}

struct ExternalUiTexture {
    bind_group: wgpu::BindGroup,
    size: [u32; 2],
    source_ptr: usize,
}

impl ExternalUiTexture {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiRendererFrameStats {
    pub upload_ms: f32,
    pub draw_ms: f32,
}

#[derive(Debug, Clone)]
struct UiDrawBatch {
    texture_id: egui::TextureId,
    scissor_x: u32,
    scissor_y: u32,
    scissor_width: u32,
    scissor_height: u32,
    first_index: u32,
    index_count: u32,
}

pub struct UiRenderer {
    pipeline: UiPipeline,
    textures: UiTextureStore,
    external_textures: HashMap<u64, ExternalUiTexture>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    staging_vertices: Vec<UiVertex>,
    staging_indices: Vec<u32>,
    draw_batches: Vec<UiDrawBatch>,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let pipeline = UiPipeline::new(device, target_format);
        let textures = UiTextureStore::new(
            device,
            queue,
            pipeline.bind_group_layout(),
            pipeline.sampler(),
            pipeline.uniform_buffer(),
        );
        let initial_vertex_capacity = 16 * 1024usize;
        let initial_index_capacity = 32 * 1024usize;
        Self {
            pipeline,
            textures,
            external_textures: HashMap::new(),
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Vertex Buffer Persistent"),
                size: (initial_vertex_capacity * std::mem::size_of::<UiVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Index Buffer Persistent"),
                size: (initial_index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            vertex_capacity: initial_vertex_capacity,
            index_capacity: initial_index_capacity,
            staging_vertices: Vec::with_capacity(initial_vertex_capacity),
            staging_indices: Vec::with_capacity(initial_index_capacity),
            draw_batches: Vec::new(),
        }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: &egui::TexturesDelta,
    ) {
        self.textures.update_textures(
            device,
            queue,
            delta,
            self.pipeline.bind_group_layout(),
            self.pipeline.sampler(),
            self.pipeline.uniform_buffer(),
        );
    }

    pub fn update_external_textures(
        &mut self,
        device: &wgpu::Device,
        inputs: &[ExternalTextureInput],
    ) {
        let mut keep_ids = HashSet::with_capacity(inputs.len());

        for input in inputs {
            keep_ids.insert(input.id);
            let needs_new = self
                .external_textures
                .get(&input.id)
                .map(|entry| entry.size != input.size || entry.source_ptr != input.source_ptr)
                .unwrap_or(true);

            if needs_new {
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("UI External Texture Bind Group"),
                    layout: self.pipeline.bind_group_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&input.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(self.pipeline.sampler()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.pipeline.uniform_buffer().as_entire_binding(),
                        },
                    ],
                });

                self.external_textures.insert(
                    input.id,
                    ExternalUiTexture {
                        bind_group,
                        size: input.size,
                        source_ptr: input.source_ptr,
                    },
                );
            }
        }

        self.external_textures.retain(|id, _| keep_ids.contains(id));
    }

    fn ensure_mesh_capacity(
        &mut self,
        device: &wgpu::Device,
        required_vertices: usize,
        required_indices: usize,
    ) {
        if required_vertices > self.vertex_capacity {
            self.vertex_capacity = required_vertices.next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Vertex Buffer Persistent"),
                size: (self.vertex_capacity * std::mem::size_of::<UiVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if required_indices > self.index_capacity {
            self.index_capacity = required_indices.next_power_of_two();
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Index Buffer Persistent"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
    }

    fn build_draw_batches(
        &mut self,
        pixels_per_point: f32,
        target_size: glam::UVec2,
        clipped_primitives: &[ClippedPrimitive],
    ) {
        self.staging_vertices.clear();
        self.staging_indices.clear();
        self.draw_batches.clear();

        for ClippedPrimitive {
            clip_rect,
            primitive,
        } in clipped_primitives
        {
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

            let vertex_base = self.staging_vertices.len() as u32;
            self.staging_vertices
                .extend(mesh.vertices.iter().map(|vertex| UiVertex {
                    pos: [vertex.pos.x, vertex.pos.y],
                    uv: [vertex.uv.x, vertex.uv.y],
                    color: vertex.color.to_array(),
                }));

            let first_index = self.staging_indices.len() as u32;
            self.staging_indices
                .extend(mesh.indices.iter().map(|index| *index as u32 + vertex_base));

            self.draw_batches.push(UiDrawBatch {
                texture_id: mesh.texture_id,
                scissor_x: clip_min_x.round() as u32,
                scissor_y: clip_min_y.round() as u32,
                scissor_width: scissor_width.round() as u32,
                scissor_height: scissor_height.round() as u32,
                first_index,
                index_count: mesh.indices.len() as u32,
            });
        }
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
    ) -> UiRendererFrameStats {
        self.build_draw_batches(pixels_per_point, target_size, clipped_primitives);
        self.ensure_mesh_capacity(
            device,
            self.staging_vertices.len(),
            self.staging_indices.len(),
        );

        let upload_start = Instant::now();
        if !self.staging_vertices.is_empty() {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(self.staging_vertices.as_slice()),
            );
        }
        if !self.staging_indices.is_empty() {
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(self.staging_indices.as_slice()),
            );
        }
        let upload_ms = upload_start.elapsed().as_secs_f32() * 1000.0;

        self.pipeline.ensure_target_format(device, target_format);
        self.pipeline.update_uniforms(queue, target_size);

        let draw_start = Instant::now();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for batch in &self.draw_batches {
            let bind_group = match batch.texture_id {
                egui::TextureId::User(id) => self
                    .external_textures
                    .get(&id)
                    .map(|texture| texture.bind_group())
                    .unwrap_or_else(|| self.textures.fallback().bind_group()),
                _ => self.textures.get(batch.texture_id).bind_group(),
            };
            render_pass.set_scissor_rect(
                batch.scissor_x,
                batch.scissor_y,
                batch.scissor_width,
                batch.scissor_height,
            );
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw_indexed(
                batch.first_index..batch.first_index + batch.index_count,
                0,
                0..1,
            );
        }

        UiRendererFrameStats {
            upload_ms,
            draw_ms: draw_start.elapsed().as_secs_f32() * 1000.0,
        }
    }
}
