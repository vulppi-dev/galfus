use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::core::image::ImageBuffer;
use crate::core::realm::RealmId;
use crate::core::ui::image_async::UiImageAsyncManager;
use crate::core::ui::renderer::ExternalTextureInput;
use crate::core::ui::types::{UiDocumentId, UiImageId, UiNodeId, UiThemeId};
use egui::Context;
pub use vulfram_ui::{UiDocument, UiNodeEntry, UiThemeState};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UiRealmState {
    pub realm_id: RealmId,
    pub last_frame_index: u64,
    pub context: Context,
    pub pixels_per_point: f32,
    pub modifiers: egui::Modifiers,
    pub pending_events: Vec<egui::Event>,
    pub last_pointer_pos: Option<egui::Pos2>,
    pub profile: UiFrameProfile,
    pub tessellation_cache: Option<UiTessellationCache>,
    pub needs_repaint: bool,
}

#[allow(dead_code)]
impl UiRealmState {
    pub fn new(realm_id: RealmId) -> Self {
        let context = Context::default();
        context.set_fonts(egui::FontDefinitions::default());
        Self {
            realm_id,
            last_frame_index: 0,
            context,
            pixels_per_point: 1.0,
            modifiers: egui::Modifiers::default(),
            pending_events: Vec::new(),
            last_pointer_pos: None,
            profile: UiFrameProfile::default(),
            tessellation_cache: None,
            needs_repaint: false,
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

    pub fn push_event(&mut self, event: egui::Event) {
        self.pending_events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<egui::Event> {
        std::mem::take(&mut self.pending_events)
    }
}

#[derive(Debug)]
pub struct UiState {
    pub realms: HashMap<RealmId, UiRealmState>,
    pub themes: HashMap<UiThemeId, UiThemeState>,
    pub documents: HashMap<UiDocumentId, UiDocument>,
    pub input_buffers: HashMap<(UiDocumentId, UiNodeId), String>,
    pub bool_values: HashMap<(UiDocumentId, UiNodeId), bool>,
    pub number_values: HashMap<(UiDocumentId, UiNodeId), f64>,
    pub selection_values: HashMap<(UiDocumentId, UiNodeId), String>,
    pub layout_rects: HashMap<(UiDocumentId, UiNodeId), glam::Vec4>,
    pub images: HashMap<UiImageId, UiImageRecord>,
    pub image_async: UiImageAsyncManager,
    pub external_textures: HashMap<u64, [u32; 2]>,
    pub target_size_requests: HashMap<u64, glam::UVec2>,
    pub animations: HashMap<UiAnimKey, UiAnimState>,
    pub split_ratios: HashMap<(UiDocumentId, UiNodeId), f32>,
    pub node_open_state: HashMap<(UiDocumentId, UiNodeId), bool>,
    pub area_positions: HashMap<(UiDocumentId, UiNodeId), glam::Vec2>,
    pub scene_state: HashMap<(UiDocumentId, UiNodeId), UiSceneState>,
    pub debug: UiDebugState,
    pub focus_by_window: HashMap<u32, RealmId>,
    pub focus_document_by_window: HashMap<u32, UiDocumentId>,
    pub focus_node_by_window: HashMap<u32, UiNodeId>,
    pub capture_by_window: HashMap<u32, (RealmId, UiDocumentId, UiNodeId)>,
    pub input_scratch: UiInputScratch,
    pub external_input_cache: HashMap<RealmId, UiExternalInputCache>,
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

    pub fn remove_document(&mut self, document_id: UiDocumentId) -> bool {
        let existed = self.documents.remove(&document_id).is_some();
        if !existed {
            return false;
        }

        self.input_buffers
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.bool_values
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.number_values
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.selection_values
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.layout_rects
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.animations
            .retain(|key, _| key.document_id != document_id);
        self.split_ratios
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.node_open_state
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.area_positions
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.scene_state
            .retain(|(entry_document_id, _), _| *entry_document_id != document_id);
        self.focus_document_by_window
            .retain(|_, focus_document_id| *focus_document_id != document_id);
        self.focus_node_by_window
            .retain(|window_id, _| self.focus_document_by_window.contains_key(window_id));
        self.capture_by_window
            .retain(|_, (_, capture_document_id, _)| *capture_document_id != document_id);

        true
    }

    pub fn remove_realm(&mut self, realm_id: RealmId) {
        self.realms.remove(&realm_id);
        self.external_input_cache.remove(&realm_id);
        self.focus_by_window
            .retain(|_, focus_realm_id| *focus_realm_id != realm_id);
        self.capture_by_window
            .retain(|_, (capture_realm_id, _, _)| *capture_realm_id != realm_id);

        let mut docs_to_remove = Vec::new();
        for (document_id, document) in &self.documents {
            if document.realm_id == realm_id {
                docs_to_remove.push(*document_id);
            }
        }
        for document_id in docs_to_remove {
            self.remove_document(document_id);
        }
        self.focus_document_by_window
            .retain(|_, document_id| self.documents.contains_key(document_id));
        self.focus_node_by_window.retain(|window_id, node_id| {
            let Some(document_id) = self.focus_document_by_window.get(window_id) else {
                return false;
            };
            self.documents
                .get(document_id)
                .map(|document| document.nodes.contains_key(node_id))
                .unwrap_or(false)
        });
        self.capture_by_window
            .retain(|_, (_, document_id, node_id)| {
                self.documents
                    .get(document_id)
                    .map(|document| document.nodes.contains_key(node_id))
                    .unwrap_or(false)
            });
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            realms: HashMap::new(),
            themes: HashMap::new(),
            documents: HashMap::new(),
            input_buffers: HashMap::new(),
            bool_values: HashMap::new(),
            number_values: HashMap::new(),
            selection_values: HashMap::new(),
            layout_rects: HashMap::new(),
            images: HashMap::new(),
            image_async: UiImageAsyncManager::new(),
            external_textures: HashMap::new(),
            target_size_requests: HashMap::new(),
            animations: HashMap::new(),
            split_ratios: HashMap::new(),
            node_open_state: HashMap::new(),
            area_positions: HashMap::new(),
            scene_state: HashMap::new(),
            debug: UiDebugState::default(),
            focus_by_window: HashMap::new(),
            focus_document_by_window: HashMap::new(),
            focus_node_by_window: HashMap::new(),
            capture_by_window: HashMap::new(),
            input_scratch: UiInputScratch::default(),
            external_input_cache: HashMap::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiFrameProfile {
    pub layout_ms: f32,
    pub tessellate_ms: f32,
    pub upload_ms: f32,
    pub draw_ms: f32,
    pub input_routing_ms: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugState {
    pub enabled: bool,
    pub show_bounds: bool,
    pub show_ids: bool,
    pub show_profile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAnimProperty {
    Opacity,
    TranslateY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiAnimKey {
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
    pub property: UiAnimProperty,
}

impl Hash for UiAnimKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.document_id.hash(state);
        self.node_id.hash(state);
        self.property.hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiAnimState {
    pub start_time: f64,
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub finished: bool,
    pub last_value: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiSceneState {
    pub pan: glam::Vec2,
    pub zoom: f32,
}

#[derive(Debug, Clone)]
pub struct UiTessellationCache {
    pub shapes_hash: u64,
    pub pixels_per_point: f32,
    pub clipped: Arc<[egui::ClippedPrimitive]>,
}

#[derive(Debug, Default)]
pub struct UiInputScratch {
    pub pointer_updates: Vec<(RealmId, egui::Event)>,
    pub modifier_updates: Vec<(RealmId, egui::Modifiers)>,
    pub focus_updates: Vec<(u32, RealmId, u32)>,
    pub pointer_pos_updates: Vec<(RealmId, Option<egui::Pos2>)>,
}

#[derive(Clone, Default)]
pub struct UiExternalInputCache {
    pub signature: u64,
    pub inputs: Vec<ExternalTextureInput>,
}

impl std::fmt::Debug for UiExternalInputCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiExternalInputCache")
            .field("signature", &self.signature)
            .field("inputs_len", &self.inputs.len())
            .finish()
    }
}

#[allow(dead_code)]
pub struct UiImageRecord {
    pub label: Option<String>,
    pub image: ImageBuffer,
    pub size: [u32; 2],
    pub texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for UiImageRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiImageRecord")
            .field("label", &self.label)
            .field("size", &self.size)
            .finish()
    }
}
