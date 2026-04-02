use std::collections::HashMap;
use std::hash::Hash;

pub fn texture_format_texel_bytes(format: wgpu::TextureFormat) -> u64 {
    match format {
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
        wgpu::TextureFormat::Depth32Float => 4,
        wgpu::TextureFormat::Rgba16Float => 8,
        _ => 0,
    }
}

pub fn estimate_texture_bytes(
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> u64 {
    let texel_bytes = texture_format_texel_bytes(format);
    if texel_bytes == 0 {
        return 0;
    }
    (size.width as u64)
        .saturating_mul(size.height as u64)
        .saturating_mul(size.depth_or_array_layers as u64)
        .saturating_mul(sample_count.max(1) as u64)
        .saturating_mul(texel_bytes)
}

#[derive(Debug, Clone)]
pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
}

impl RenderTarget {
    pub fn new(device: &wgpu::Device, size: wgpu::Extent3d, format: wgpu::TextureFormat) -> Self {
        Self::new_with_samples(device, size, format, 1)
    }

    pub fn new_with_samples(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Camera RenderTarget"),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            format,
            sample_count,
        }
    }

    pub fn estimated_bytes(&self) -> u64 {
        estimate_texture_bytes(self.texture.size(), self.format, self.sample_count)
    }
}

pub fn ensure_render_target(
    device: &wgpu::Device,
    target: &mut Option<RenderTarget>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) {
    let needs_target = match target.as_ref() {
        Some(existing) => {
            let size = existing.texture.size();
            size.width != width || size.height != height || existing.format != format
        }
        None => true,
    };

    if needs_target {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        *target = Some(RenderTarget::new(device, size, format));
    }
}

pub fn ensure_surface_target<'a, K>(
    device: &wgpu::Device,
    surface_targets: &'a mut HashMap<K, RenderTarget>,
    surface_id: K,
    size: glam::UVec2,
    format: wgpu::TextureFormat,
) -> &'a RenderTarget
where
    K: Eq + Hash + Copy,
{
    let size = glam::UVec2::new(size.x.max(1), size.y.max(1));
    let needs_target = match surface_targets.get(&surface_id) {
        Some(existing) => {
            let tex_size = existing.texture.size();
            tex_size.width != size.x || tex_size.height != size.y || existing.format != format
        }
        None => true,
    };

    if needs_target {
        let extent = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        surface_targets.insert(surface_id, RenderTarget::new(device, extent, format));
    }

    surface_targets
        .get(&surface_id)
        .expect("surface target missing after ensure")
}
