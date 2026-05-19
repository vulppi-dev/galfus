pub mod input;

use crate::core::realm::RealmId;

#[derive(Debug, Default)]
pub struct UiState {
    pub external_textures: std::collections::HashMap<u32, glam::UVec2>,
    pub target_size_requests: std::collections::HashMap<u64, glam::UVec2>,
}

impl UiState {
    pub fn ensure_realm(&mut self, _realm_id: RealmId) {}

    pub fn remove_realm(&mut self, _realm_id: RealmId) {}
}

#[derive(Debug, Default)]
pub struct UiRenderer;

impl UiRenderer {
    pub fn estimated_gpu_bytes(&self) -> u64 {
        0
    }
}
