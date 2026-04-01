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
