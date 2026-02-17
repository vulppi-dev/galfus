use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::image::ImageBuffer;
use crate::core::realm::RealmId;
use crate::core::ui::image_async::UiImageAsyncManager;
use crate::core::ui::types::{
    UiDocumentId, UiImageId, UiNode, UiNodeId, UiNodeProps, UiThemeId, UiThemeValue,
};
use egui::Context;

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
        self.focus_document_by_window.retain(|_, document_id| {
            self.documents.contains_key(document_id)
        });
        self.focus_node_by_window.retain(|window_id, node_id| {
            let Some(document_id) = self.focus_document_by_window.get(window_id) else {
                return false;
            };
            self.documents
                .get(document_id)
                .map(|document| document.nodes.contains_key(node_id))
                .unwrap_or(false)
        });
        self.capture_by_window.retain(|_, (_, document_id, node_id)| {
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
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiFrameProfile {
    pub layout_ms: f32,
    pub tessellate_ms: f32,
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
    pub clipped: Vec<egui::ClippedPrimitive>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UiThemeState {
    pub version: u32,
    pub data: HashMap<String, UiThemeValue>,
}

#[derive(Debug, Clone)]
pub struct UiNodeEntry {
    pub node: UiNode,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UiDocument {
    pub document_id: UiDocumentId,
    pub realm_id: RealmId,
    pub rect: glam::Vec4,
    pub theme_id: Option<UiThemeId>,
    pub last_theme_version: Option<u32>,
    pub version: u64,
    pub nodes: HashMap<UiNodeId, UiNodeEntry>,
    pub root_children: Vec<UiNodeId>,
    pub layout_dirty: bool,
    pub ordered_root: Vec<UiNodeId>,
    pub ordered_children: HashMap<UiNodeId, Vec<UiNodeId>>,
}

impl UiDocument {
    pub fn new(document_id: UiDocumentId, realm_id: RealmId, rect: glam::Vec4) -> Self {
        Self {
            document_id,
            realm_id,
            rect,
            theme_id: None,
            last_theme_version: None,
            version: 0,
            nodes: HashMap::new(),
            root_children: Vec::new(),
            layout_dirty: true,
            ordered_root: Vec::new(),
            ordered_children: HashMap::new(),
        }
    }

    pub fn add_node(
        &mut self,
        parent: Option<UiNodeId>,
        node: UiNode,
        index: Option<u32>,
    ) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("UiNode {} already exists", node.id));
        }
        if let Some(parent_id) = parent {
            if !self.nodes.contains_key(&parent_id) {
                return Err(format!("Parent UiNode {} not found", parent_id));
            }
        }
        let entry = UiNodeEntry {
            node: node.clone(),
            parent,
            children: Vec::new(),
        };
        self.nodes.insert(node.id, entry);
        self.insert_child(parent, node.id, index);
        self.layout_dirty = true;
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: UiNodeId) -> Result<(), String> {
        if !self.nodes.contains_key(&node_id) {
            return Err(format!("UiNode {} not found", node_id));
        }
        self.detach_child(node_id);
        self.remove_subtree(node_id);
        self.layout_dirty = true;
        Ok(())
    }

    pub fn clear_children(&mut self, parent: Option<UiNodeId>) -> Result<(), String> {
        let children = match parent {
            Some(parent_id) => {
                let entry = self
                    .nodes
                    .get(&parent_id)
                    .ok_or_else(|| format!("UiNode {} not found", parent_id))?;
                entry.children.clone()
            }
            None => self.root_children.clone(),
        };
        for child in children {
            self.remove_subtree(child);
        }
        if let Some(parent_id) = parent {
            if let Some(entry) = self.nodes.get_mut(&parent_id) {
                entry.children.clear();
            }
        } else {
            self.root_children.clear();
        }
        self.layout_dirty = true;
        Ok(())
    }

    pub fn set_props(&mut self, node_id: UiNodeId, props: UiNodeProps) -> Result<(), String> {
        let entry = self
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| format!("UiNode {} not found", node_id))?;
        entry.node.props = props;
        self.layout_dirty = true;
        Ok(())
    }

    pub fn move_node(
        &mut self,
        node_id: UiNodeId,
        new_parent: Option<UiNodeId>,
        index: Option<u32>,
    ) -> Result<(), String> {
        if !self.nodes.contains_key(&node_id) {
            return Err(format!("UiNode {} not found", node_id));
        }
        if let Some(parent_id) = new_parent {
            if !self.nodes.contains_key(&parent_id) {
                return Err(format!("Parent UiNode {} not found", parent_id));
            }
        }
        self.detach_child(node_id);
        if let Some(entry) = self.nodes.get_mut(&node_id) {
            entry.parent = new_parent;
        }
        self.insert_child(new_parent, node_id, index);
        self.layout_dirty = true;
        Ok(())
    }

    fn insert_child(&mut self, parent: Option<UiNodeId>, node_id: UiNodeId, index: Option<u32>) {
        let list = match parent {
            Some(parent_id) => {
                &mut self
                    .nodes
                    .get_mut(&parent_id)
                    .expect("parent checked")
                    .children
            }
            None => &mut self.root_children,
        };
        let insert_index = index
            .map(|value| value as usize)
            .filter(|value| *value <= list.len())
            .unwrap_or(list.len());
        list.insert(insert_index, node_id);
    }

    fn detach_child(&mut self, node_id: UiNodeId) {
        let parent = self.nodes.get(&node_id).and_then(|entry| entry.parent);
        let list = match parent {
            Some(parent_id) => self
                .nodes
                .get_mut(&parent_id)
                .map(|entry| &mut entry.children),
            None => Some(&mut self.root_children),
        };
        if let Some(list) = list {
            if let Some(pos) = list.iter().position(|child| *child == node_id) {
                list.remove(pos);
            }
        }
    }

    fn remove_subtree(&mut self, node_id: UiNodeId) {
        let children = match self.nodes.get(&node_id) {
            Some(entry) => entry.children.clone(),
            None => return,
        };
        for child in children {
            self.remove_subtree(child);
        }
        self.nodes.remove(&node_id);
    }

    pub fn ensure_layout_cache(&mut self) {
        if !self.layout_dirty {
            return;
        }
        self.ordered_root = self.sort_children(&self.root_children);
        self.ordered_children.clear();
        for (node_id, entry) in &self.nodes {
            let ordered = self.sort_children(&entry.children);
            self.ordered_children.insert(*node_id, ordered);
        }
        self.layout_dirty = false;
    }

    fn sort_children(&self, children: &[UiNodeId]) -> Vec<UiNodeId> {
        let mut ordered: Vec<(usize, UiNodeId, i32)> = children
            .iter()
            .enumerate()
            .map(|(index, node_id)| {
                let z = self
                    .nodes
                    .get(node_id)
                    .and_then(|entry| entry.node.z_index)
                    .unwrap_or(0);
                (index, *node_id, z)
            })
            .collect();
        ordered.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));
        ordered.into_iter().map(|(_, node_id, _)| node_id).collect()
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
