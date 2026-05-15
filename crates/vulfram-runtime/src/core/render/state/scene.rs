use crate::core::resources::{
    CameraRecord, ForwardAtlasEntry, LightRecord, ModelRecord, ShaderMaterialRecord, TextureRecord,
};
use std::collections::HashMap;

/// Holds the actual scene data to be rendered
#[derive(Default)]
pub struct RenderScene {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,
    pub lights: HashMap<u32, LightRecord>,
    pub materials: HashMap<u32, ShaderMaterialRecord>,
    pub textures: HashMap<u32, TextureRecord>,
    pub forward_atlas_entries: HashMap<u32, ForwardAtlasEntry>,
}
