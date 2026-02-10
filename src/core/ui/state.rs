use std::collections::HashMap;

use crate::core::realm::RealmId;
use egui::Context;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UiRealmState {
    pub realm_id: RealmId,
    pub last_frame_index: u64,
    pub context: Context,
    pub pixels_per_point: f32,
}

#[allow(dead_code)]
impl UiRealmState {
    pub fn new(realm_id: RealmId) -> Self {
        Self {
            realm_id,
            last_frame_index: 0,
            context: Context::default(),
            pixels_per_point: 1.0,
        }
    }

    pub fn realm_id(&self) -> RealmId {
        self.realm_id
    }

    pub fn last_frame_index(&self) -> u64 {
        self.last_frame_index
    }

    pub fn set_last_frame_index(&mut self, frame_index: u64) {
        self.last_frame_index = frame_index;
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub realms: HashMap<RealmId, UiRealmState>,
}

impl UiState {
    pub fn ensure_realm(&mut self, realm_id: RealmId) {
        self.realms
            .entry(realm_id)
            .or_insert_with(|| UiRealmState::new(realm_id));
    }

    pub fn realm_mut(&mut self, realm_id: RealmId) -> Option<&mut UiRealmState> {
        self.realms.get_mut(&realm_id)
    }

    pub fn remove_realm(&mut self, realm_id: RealmId) {
        self.realms.remove(&realm_id);
    }
}
