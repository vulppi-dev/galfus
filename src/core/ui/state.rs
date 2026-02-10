use std::collections::HashMap;

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

#[derive(Debug)]
pub struct UiState {
    pub realms: HashMap<RealmId, UiRealmState>,
    pub themes: HashMap<UiThemeId, UiThemeState>,
    pub documents: HashMap<UiDocumentId, UiDocument>,
    pub images: HashMap<UiImageId, UiImageRecord>,
    pub image_async: UiImageAsyncManager,
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

impl Default for UiState {
    fn default() -> Self {
        Self {
            realms: HashMap::new(),
            themes: HashMap::new(),
            documents: HashMap::new(),
            images: HashMap::new(),
            image_async: UiImageAsyncManager::new(),
        }
    }
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
    pub version: u64,
    pub nodes: HashMap<UiNodeId, UiNodeEntry>,
    pub root_children: Vec<UiNodeId>,
}

impl UiDocument {
    pub fn new(document_id: UiDocumentId, realm_id: RealmId, rect: glam::Vec4) -> Self {
        Self {
            document_id,
            realm_id,
            rect,
            theme_id: None,
            version: 0,
            nodes: HashMap::new(),
            root_children: Vec::new(),
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
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: UiNodeId) -> Result<(), String> {
        if !self.nodes.contains_key(&node_id) {
            return Err(format!("UiNode {} not found", node_id));
        }
        self.detach_child(node_id);
        self.remove_subtree(node_id);
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
        Ok(())
    }

    pub fn set_props(&mut self, node_id: UiNodeId, props: UiNodeProps) -> Result<(), String> {
        let entry = self
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| format!("UiNode {} not found", node_id))?;
        entry.node.props = props;
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
        Ok(())
    }

    fn insert_child(&mut self, parent: Option<UiNodeId>, node_id: UiNodeId, index: Option<u32>) {
        let list = match parent {
            Some(parent_id) => &mut self
                .nodes
                .get_mut(&parent_id)
                .expect("parent checked")
                .children,
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
