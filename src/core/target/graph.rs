use std::collections::{HashMap, HashSet};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::target::graph_hash::{hash_entries, hash_targets_layers_and_realms};
use crate::core::target::{TargetId, TargetLayerState, TargetState};
#[allow(unused_imports)]
pub use vulfram_scene_core::{TargetEdge, TargetGraphDiff, TargetGraphPlan};

#[derive(Debug, Default)]
pub struct TargetGraphPlanner;

impl TargetGraphPlanner {
    pub fn build_plan(
        &self,
        targets: &HashMap<TargetId, TargetState>,
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &RealmTable,
    ) -> TargetGraphPlan {
        let target_semantics = targets
            .iter()
            .map(|(target_id, target)| (*target_id, (target.kind, target.window_id)))
            .collect();
        let realm_ids = realms.entries.keys().copied().collect();
        vulfram_scene_core::TargetGraphPlanner.build_plan(&target_semantics, layers, &realm_ids)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphCache {
    pub last_hash: u64,
    pub last_target_hashes: HashMap<TargetId, u64>,
    pub last_layer_hashes: HashMap<(u32, TargetId), u64>,
    pub last_realm_hashes: HashMap<RealmId, u64>,
    pub last_plan: TargetGraphPlan,
    pub last_diff: Option<TargetGraphDiff>,
}

impl TargetGraphCache {
    pub fn update(
        &mut self,
        targets: &HashMap<TargetId, TargetState>,
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &RealmTable,
    ) -> Option<&TargetGraphDiff> {
        let current_hash = hash_targets_layers_and_realms(targets, layers, realms);
        if current_hash == self.last_hash {
            self.last_diff = None;
            return None;
        }

        let (target_hashes, layer_hashes, realm_hashes) = hash_entries(targets, layers, realms);
        let diff = diff_targets_layers_and_realms(
            &self.last_target_hashes,
            &self.last_layer_hashes,
            &self.last_realm_hashes,
            &target_hashes,
            &layer_hashes,
            &realm_hashes,
            layers,
        );
        self.last_target_hashes = target_hashes;
        self.last_layer_hashes = layer_hashes;
        self.last_realm_hashes = realm_hashes;
        self.last_hash = current_hash;
        if diff.plan_dirty {
            self.last_plan = TargetGraphPlanner.build_plan(targets, layers, realms);
        }
        self.last_diff = Some(diff);
        self.last_diff.as_ref()
    }

    pub fn prune_dead_entries(
        &mut self,
        targets: &HashMap<TargetId, TargetState>,
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &RealmTable,
    ) {
        self.last_target_hashes
            .retain(|target_id, _| targets.contains_key(target_id));
        self.last_layer_hashes
            .retain(|layer_key, _| layers.contains_key(layer_key));
        self.last_realm_hashes
            .retain(|realm_id, _| realms.entries.contains_key(realm_id));
    }
}

fn diff_targets_layers_and_realms(
    previous_targets: &HashMap<TargetId, u64>,
    previous_layers: &HashMap<(u32, TargetId), u64>,
    previous_realms: &HashMap<RealmId, u64>,
    targets: &HashMap<TargetId, u64>,
    layers: &HashMap<(u32, TargetId), u64>,
    realms: &HashMap<RealmId, u64>,
    layer_states: &HashMap<(u32, TargetId), TargetLayerState>,
) -> TargetGraphDiff {
    let mut diff = TargetGraphDiff::default();

    for (target_id, state) in targets {
        match previous_targets.get(target_id) {
            None => diff.added_targets.push(*target_id),
            Some(prev) if prev != state => diff.updated_targets.push(*target_id),
            _ => {}
        }
    }
    for target_id in previous_targets.keys() {
        if !targets.contains_key(target_id) {
            diff.removed_targets.push(*target_id);
        }
    }

    for (layer_key, state) in layers {
        match previous_layers.get(layer_key) {
            None => diff.added_layers.push(*layer_key),
            Some(prev) if prev != state => diff.updated_layers.push(*layer_key),
            _ => {}
        }
    }
    for layer_key in previous_layers.keys() {
        if !layers.contains_key(layer_key) {
            diff.removed_layers.push(*layer_key);
        }
    }

    let mut realms_changed: HashSet<RealmId> = HashSet::new();
    for (realm_id, state) in realms {
        match previous_realms.get(realm_id) {
            None => {
                realms_changed.insert(*realm_id);
            }
            Some(prev) if prev != state => {
                realms_changed.insert(*realm_id);
            }
            _ => {}
        }
    }
    for realm_id in previous_realms.keys() {
        if !realms.contains_key(realm_id) {
            realms_changed.insert(*realm_id);
        }
    }

    if !realms_changed.is_empty() {
        for ((realm_id, target_id), _layer) in layer_states {
            if realms_changed.contains(&RealmId(*realm_id)) {
                diff.updated_layers.push((*realm_id, *target_id));
            }
        }
    }

    diff.added_targets.sort_by_key(|id| id.0);
    diff.removed_targets.sort_by_key(|id| id.0);
    diff.updated_targets.sort_by_key(|id| id.0);
    diff.added_layers.sort_by_key(|(realm, id)| (*realm, id.0));
    diff.removed_layers
        .sort_by_key(|(realm, id)| (*realm, id.0));
    diff.updated_layers
        .sort_by_key(|(realm, id)| (*realm, id.0));
    diff.updated_layers.dedup();

    let mut dirty_targets: HashSet<TargetId> = HashSet::new();
    for target_id in &diff.added_targets {
        dirty_targets.insert(*target_id);
    }
    for target_id in &diff.removed_targets {
        dirty_targets.insert(*target_id);
    }
    for target_id in &diff.updated_targets {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in &diff.added_layers {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in &diff.removed_layers {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in &diff.updated_layers {
        dirty_targets.insert(*target_id);
    }
    diff.dirty_targets = dirty_targets.into_iter().collect();
    diff.dirty_targets.sort_by_key(|id| id.0);
    diff.plan_dirty = !diff.added_targets.is_empty()
        || !diff.removed_targets.is_empty()
        || !diff.updated_targets.is_empty()
        || !diff.added_layers.is_empty()
        || !diff.removed_layers.is_empty()
        || !realms_changed.is_empty();

    diff
}
