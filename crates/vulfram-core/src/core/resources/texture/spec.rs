use glam::Vec4;

#[derive(Debug, Clone)]
pub struct TextureRecord {
    pub label: Option<String>,
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

#[derive(Debug, Clone)]
pub struct TargetTextureBinding {
    pub target_id: crate::core::target::TargetId,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ForwardAtlasEntry {
    pub label: Option<String>,
    pub uv_scale_bias: Vec4,
    pub layer: u32,
}
