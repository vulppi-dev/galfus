use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use vulfram_types::RealmId;

use crate::{UiDocumentId, UiNode, UiNodeId, UiNodeProps, UiOp, UiThemeId, UiThemeValue};

#[derive(Debug, Clone)]
pub struct UiThemeState {
    pub version: u32,
    pub data: HashMap<String, UiThemeValue>,
    pub font_data: HashMap<String, Vec<u8>>,
    pub font_families: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct UiNodeEntry {
    pub node: UiNode,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
}

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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct UiApplyOpsResult {
    pub message: String,
    pub version: Option<u64>,
    pub removed_nodes: Vec<UiNodeId>,
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
        if let Some(parent_id) = parent
            && !self.nodes.contains_key(&parent_id)
        {
            return Err(format!("Parent UiNode {} not found", parent_id));
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
        if let Some(parent_id) = new_parent
            && !self.nodes.contains_key(&parent_id)
        {
            return Err(format!("Parent UiNode {} not found", parent_id));
        }
        self.detach_child(node_id);
        if let Some(entry) = self.nodes.get_mut(&node_id) {
            entry.parent = new_parent;
        }
        self.insert_child(new_parent, node_id, index);
        self.layout_dirty = true;
        Ok(())
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

    pub fn apply_ops(
        &mut self,
        version: u64,
        ops: &[UiOp],
    ) -> Result<UiApplyOpsResult, UiApplyOpsResult> {
        if version <= self.version {
            return Err(UiApplyOpsResult {
                message: format!(
                    "UiDocument {} version mismatch (current={}, incoming={})",
                    self.document_id, self.version, version
                ),
                version: Some(self.version),
                removed_nodes: Vec::new(),
            });
        }

        let current_version = self.version;
        let mut undo_log: Vec<UndoAction> = Vec::with_capacity(ops.len());
        let mut removed_nodes: HashSet<UiNodeId> = HashSet::new();

        for op in ops {
            let result = match op {
                UiOp::Add {
                    parent,
                    node,
                    index,
                } => (|| -> Result<(), String> {
                    self.add_node(*parent, node.clone(), *index)?;
                    undo_log.push(UndoAction::Add { node_id: node.id });
                    Ok(())
                })(),
                UiOp::Remove { node_id } => (|| -> Result<(), String> {
                    let snapshot = snapshot_subtree(self, *node_id)?;
                    removed_nodes.extend(snapshot.entries.keys().copied());
                    self.remove_node(*node_id)?;
                    undo_log.push(UndoAction::Remove { snapshot });
                    Ok(())
                })(),
                UiOp::Clear { parent } => (|| -> Result<(), String> {
                    let children = children_of(self, *parent)?;
                    let mut snapshots: Vec<SubtreeSnapshot> = Vec::with_capacity(children.len());
                    for child in children {
                        let snapshot = snapshot_subtree(self, child)?;
                        removed_nodes.extend(snapshot.entries.keys().copied());
                        snapshots.push(snapshot);
                    }
                    self.clear_children(*parent)?;
                    undo_log.push(UndoAction::Clear { snapshots });
                    Ok(())
                })(),
                UiOp::Set { node_id, props } => (|| -> Result<(), String> {
                    let old_props = self
                        .nodes
                        .get(node_id)
                        .ok_or_else(|| format!("UiNode {} not found", node_id))?
                        .node
                        .props
                        .clone();
                    self.set_props(*node_id, props.clone())?;
                    undo_log.push(UndoAction::Set {
                        node_id: *node_id,
                        old_props,
                    });
                    Ok(())
                })(),
                UiOp::Move {
                    node_id,
                    new_parent,
                    index,
                } => (|| -> Result<(), String> {
                    let old_parent = self
                        .nodes
                        .get(node_id)
                        .ok_or_else(|| format!("UiNode {} not found", node_id))?
                        .parent;
                    let old_index = child_index(self, old_parent, *node_id)
                        .ok_or_else(|| format!("UiNode {} not attached to parent", node_id))?;
                    self.move_node(*node_id, *new_parent, *index)?;
                    undo_log.push(UndoAction::Move {
                        node_id: *node_id,
                        old_parent,
                        old_index,
                    });
                    Ok(())
                })(),
            };

            if let Err(message) = result {
                rollback_document_ops(self, &undo_log);
                return Err(UiApplyOpsResult {
                    message,
                    version: Some(current_version),
                    removed_nodes: Vec::new(),
                });
            }
        }

        self.version = version;
        let mut removed_nodes: Vec<UiNodeId> = removed_nodes.into_iter().collect();
        removed_nodes.sort_unstable();
        Ok(UiApplyOpsResult {
            message: "UI ops applied".into(),
            version: Some(version),
            removed_nodes,
        })
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
        if let Some(list) = list
            && let Some(pos) = list.iter().position(|child| *child == node_id)
        {
            list.remove(pos);
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

#[derive(Debug, Clone)]
struct SubtreeSnapshot {
    root_id: UiNodeId,
    parent: Option<UiNodeId>,
    index: usize,
    entries: HashMap<UiNodeId, UiNodeEntry>,
}

#[derive(Debug, Clone)]
enum UndoAction {
    Add {
        node_id: UiNodeId,
    },
    Remove {
        snapshot: SubtreeSnapshot,
    },
    Clear {
        snapshots: Vec<SubtreeSnapshot>,
    },
    Set {
        node_id: UiNodeId,
        old_props: UiNodeProps,
    },
    Move {
        node_id: UiNodeId,
        old_parent: Option<UiNodeId>,
        old_index: usize,
    },
}

fn rollback_document_ops(doc: &mut UiDocument, undo_log: &[UndoAction]) {
    for undo in undo_log.iter().rev() {
        match undo {
            UndoAction::Add { node_id } => {
                let _ = doc.remove_node(*node_id);
            }
            UndoAction::Remove { snapshot } => {
                restore_subtree(doc, snapshot);
            }
            UndoAction::Clear { snapshots } => {
                for snapshot in snapshots {
                    restore_subtree(doc, snapshot);
                }
            }
            UndoAction::Set { node_id, old_props } => {
                let _ = doc.set_props(*node_id, old_props.clone());
            }
            UndoAction::Move {
                node_id,
                old_parent,
                old_index,
            } => {
                let _ = doc.move_node(*node_id, *old_parent, Some(*old_index as u32));
            }
        }
    }
}

fn snapshot_subtree(doc: &UiDocument, node_id: UiNodeId) -> Result<SubtreeSnapshot, String> {
    let entry = doc
        .nodes
        .get(&node_id)
        .ok_or_else(|| format!("UiNode {} not found", node_id))?;
    let parent = entry.parent;
    let index = child_index(doc, parent, node_id)
        .ok_or_else(|| format!("UiNode {} not attached to parent", node_id))?;

    let mut stack = vec![node_id];
    let mut entries: HashMap<UiNodeId, UiNodeEntry> = HashMap::new();
    while let Some(current) = stack.pop() {
        let Some(current_entry) = doc.nodes.get(&current) else {
            continue;
        };
        for child in &current_entry.children {
            stack.push(*child);
        }
        entries.insert(current, current_entry.clone());
    }

    Ok(SubtreeSnapshot {
        root_id: node_id,
        parent,
        index,
        entries,
    })
}

fn restore_subtree(doc: &mut UiDocument, snapshot: &SubtreeSnapshot) {
    for (node_id, entry) in &snapshot.entries {
        doc.nodes.insert(*node_id, entry.clone());
    }
    let list = match snapshot.parent {
        Some(parent_id) => doc
            .nodes
            .get_mut(&parent_id)
            .map(|entry| &mut entry.children),
        None => Some(&mut doc.root_children),
    };
    if let Some(children) = list {
        let insert_index = snapshot.index.min(children.len());
        if !children.contains(&snapshot.root_id) {
            children.insert(insert_index, snapshot.root_id);
        }
    }
    doc.layout_dirty = true;
}

fn child_index(doc: &UiDocument, parent: Option<UiNodeId>, node_id: UiNodeId) -> Option<usize> {
    let children = match parent {
        Some(parent_id) => doc.nodes.get(&parent_id).map(|entry| &entry.children)?,
        None => &doc.root_children,
    };
    children.iter().position(|child| *child == node_id)
}

fn children_of(doc: &UiDocument, parent: Option<UiNodeId>) -> Result<Vec<UiNodeId>, String> {
    match parent {
        Some(parent_id) => Ok(doc
            .nodes
            .get(&parent_id)
            .ok_or_else(|| format!("UiNode {} not found", parent_id))?
            .children
            .clone()),
        None => Ok(doc.root_children.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiNodeKind, UiNodeProps, UiOp};

    fn text_node(node_id: UiNodeId, text: &str) -> UiNode {
        UiNode {
            id: node_id,
            kind: UiNodeKind::Text,
            props: UiNodeProps::Text {
                text: text.into(),
                size: None,
                color: None,
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index: None,
        }
    }

    #[test]
    fn ui_document_add_move_and_remove_nodes() {
        let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));

        doc.add_node(None, text_node(10, "root"), None)
            .expect("root should be added");
        doc.add_node(Some(10), text_node(11, "child"), None)
            .expect("child should be added");
        doc.move_node(11, None, Some(0)).expect("move should work");

        assert_eq!(doc.root_children, vec![11, 10]);

        doc.remove_node(10).expect("remove should work");
        assert!(!doc.nodes.contains_key(&10));
    }

    #[test]
    fn ui_document_apply_ops_rolls_back_on_error() {
        let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
        doc.add_node(None, text_node(10, "root"), None)
            .expect("root should be added");

        let result = doc.apply_ops(
            2,
            &[
                UiOp::Add {
                    parent: Some(10),
                    node: text_node(11, "child"),
                    index: None,
                },
                UiOp::Set {
                    node_id: 999,
                    props: UiNodeProps::Text {
                        text: "broken".into(),
                        size: None,
                        color: None,
                    },
                },
            ],
        );

        assert!(result.is_err());
        assert!(!doc.nodes.contains_key(&11));
        assert_eq!(doc.version, 0);
        assert_eq!(
            doc.nodes.get(&10).map(|entry| entry.children.len()),
            Some(0)
        );
    }

    #[test]
    fn ui_document_apply_ops_reports_removed_nodes() {
        let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
        doc.add_node(None, text_node(10, "root"), None)
            .expect("root should be added");
        doc.add_node(Some(10), text_node(11, "child"), None)
            .expect("child should be added");

        let result = doc
            .apply_ops(2, &[UiOp::Remove { node_id: 10 }])
            .expect("remove should work");

        assert_eq!(result.version, Some(2));
        assert_eq!(result.removed_nodes, vec![10, 11]);
        assert!(doc.nodes.is_empty());
    }

    #[test]
    fn ui_document_layout_cache_sorts_by_z_index() {
        let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
        let mut a = text_node(10, "a");
        a.z_index = Some(10);
        let mut b = text_node(11, "b");
        b.z_index = Some(-1);

        doc.add_node(None, a, None).expect("a should be added");
        doc.add_node(None, b, None).expect("b should be added");
        doc.ensure_layout_cache();

        assert_eq!(doc.ordered_root, vec![11, 10]);
    }
}
