use std::collections::HashMap;

use egui::{ImageData, TexturesDelta, TextureId};
use half::f16;

pub struct UiTexture {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    size: [u32; 2],
}

pub struct UiTextureStore {
    textures: HashMap<TextureId, UiTexture>,
    fallback: UiTexture,
}

impl UiTextureStore {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniform_buffer: &wgpu::Buffer,
    ) -> Self {
        let fallback = create_fallback_texture(device, queue, bind_group_layout, sampler, uniform_buffer);
        Self {
            textures: HashMap::new(),
            fallback,
        }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: &TexturesDelta,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniform_buffer: &wgpu::Buffer,
    ) {
        for (id, image_delta) in &delta.set {
            let image_size = match &image_delta.image {
                ImageData::Color(color) => [color.size[0] as u32, color.size[1] as u32],
                ImageData::Font(font) => [font.size[0] as u32, font.size[1] as u32],
            };

            let existing = self.textures.get(id);
            let (texture_size, origin) = if let Some(pos) = image_delta.pos {
                let origin = wgpu::Origin3d {
                    x: pos[0] as u32,
                    y: pos[1] as u32,
                    z: 0,
                };
                let target_size = existing
                    .map(|tex| tex.size)
                    .unwrap_or_else(|| [pos[0] as u32 + image_size[0], pos[1] as u32 + image_size[1]]);
                (target_size, origin)
            } else {
                (image_size, wgpu::Origin3d::ZERO)
            };

            let needs_new = existing
                .map(|tex| tex.size != texture_size)
                .unwrap_or(true);

            if needs_new {
                let texture = create_texture(
                    device,
                    bind_group_layout,
                    sampler,
                    uniform_buffer,
                    texture_size,
                );
                self.textures.insert(*id, texture);
            }

            if let Some(texture) = self.textures.get(id) {
                let bytes = match &image_delta.image {
                    ImageData::Color(color) => build_color_bytes_rgba16f(color),
                    ImageData::Font(font) => build_font_bytes_rgba16f(font),
                };

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture.texture,
                        mip_level: 0,
                        origin,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(8 * image_size[0]),
                        rows_per_image: Some(image_size[1]),
                    },
                    wgpu::Extent3d {
                        width: image_size[0],
                        height: image_size[1],
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        for id in &delta.free {
            self.textures.remove(id);
        }
    }

    pub fn get(&self, id: TextureId) -> &UiTexture {
        self.textures.get(&id).unwrap_or(&self.fallback)
    }

    pub fn fallback(&self) -> &UiTexture {
        &self.fallback
    }
}

impl UiTexture {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

fn create_texture(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    uniform_buffer: &wgpu::Buffer,
    size: [u32; 2],
) -> UiTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("UI Texture"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("UI Texture Bind Group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });

    UiTexture {
        texture,
        bind_group,
        size,
    }
}

fn create_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    uniform_buffer: &wgpu::Buffer,
) -> UiTexture {
    let texture = create_texture(device, bind_group_layout, sampler, uniform_buffer, [1, 1]);
    let mut bytes = Vec::with_capacity(8);
    push_rgba16f(&mut bytes, [1.0, 1.0, 1.0, 1.0]);
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    texture
}

fn push_rgba16f(bytes: &mut Vec<u8>, rgba: [f32; 4]) {
    for channel in rgba {
        let half = f16::from_f32(channel);
        bytes.extend_from_slice(&half.to_le_bytes());
    }
}

fn build_color_bytes_rgba16f(image: &egui::ColorImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(image.pixels.len() * 8);
    for pixel in &image.pixels {
        let [r, g, b, a] = pixel.to_array();
        let rgba = [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ];
        push_rgba16f(&mut bytes, rgba);
    }
    bytes
}

fn build_font_bytes_rgba16f(image: &egui::FontImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(image.pixels.len() * 8);
    for alpha in &image.pixels {
        let coverage = (*alpha).clamp(0.0, 1.0);
        let premultiplied = coverage.powf(0.55);
        let rgba = [premultiplied, premultiplied, premultiplied, premultiplied];
        push_rgba16f(&mut bytes, rgba);
    }
    bytes
}
