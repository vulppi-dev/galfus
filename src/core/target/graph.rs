use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::target::graph_hash::{hash_entries, hash_targets_layers_and_realms};
use crate::core::target::{TargetId, TargetKind, TargetLayerState, TargetState};
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
        let window_targets = collect_window_targets(targets);
        let layers_by_target = collect_layers_by_target(layers);
        let realm_windows = collect_realm_windows(targets, layers, realms);
        let mut edges = Vec::with_capacity(targets.len());

        for (target_id, target) in targets {
            match target.kind {
                TargetKind::Window | TargetKind::Texture => {}
                TargetKind::WidgetRealmViewport | TargetKind::RealmPlane => {
                    if let Some(parent) = infer_parent_from_layers(
                        &layers_by_target,
                        &realm_windows,
                        *target_id,
                        &window_targets,
                    ) {
                        edges.push(TargetEdge {
                            parent,
                            child: *target_id,
                        });
                    }
                }
            }
        }

        edges.sort_by_key(|edge| (edge.parent.0, edge.child.0));

        let all_targets: HashSet<TargetId> = targets.keys().copied().collect();
        let (order, cut_edges) = topo_with_soft_cuts(&all_targets, &edges);

        TargetGraphPlan {
            edges,
            order,
            cut_edges,
        }
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

fn collect_window_targets(targets: &HashMap<TargetId, TargetState>) -> HashMap<u32, TargetId> {
    let mut map: HashMap<u32, TargetId> = HashMap::new();
    for (target_id, state) in targets {
        if state.kind != TargetKind::Window {
            continue;
        }
        if let Some(window_id) = state.window_id {
            if let Some(existing) = map.get_mut(&window_id) {
                if target_id.0 < existing.0 {
                    *existing = *target_id;
                }
            } else {
                map.insert(window_id, *target_id);
            }
        }
    }
    map
}

fn infer_parent_from_layers(
    layers_by_target: &HashMap<TargetId, Vec<u32>>,
    realm_windows: &HashMap<u32, u32>,
    target_id: TargetId,
    window_targets: &HashMap<u32, TargetId>,
) -> Option<TargetId> {
    let mut chosen_window: Option<u32> = None;
    let mut chosen_realm: Option<u32> = None;

    let Some(realm_ids) = layers_by_target.get(&target_id) else {
        return None;
    };
    for layer_realm_id in realm_ids {
        let Some(realm_window_id) = realm_windows.get(layer_realm_id).copied() else {
            continue;
        };

        match chosen_window {
            None => {
                chosen_window = Some(realm_window_id);
                chosen_realm = Some(*layer_realm_id);
            }
            Some(current_window) => {
                if realm_window_id < current_window {
                    chosen_window = Some(realm_window_id);
                    chosen_realm = Some(*layer_realm_id);
                } else if realm_window_id == current_window {
                    let current_realm = chosen_realm.unwrap_or(u32::MAX);
                    if *layer_realm_id < current_realm {
                        chosen_realm = Some(*layer_realm_id);
                    }
                }
            }
        }
    }

    let window_id = chosen_window?;
    window_targets.get(&window_id).copied()
}

fn collect_realm_windows(
    targets: &HashMap<TargetId, TargetState>,
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    realms: &RealmTable,
) -> HashMap<u32, u32> {
    let mut map: HashMap<u32, u32> = HashMap::new();
    for ((realm_id, target_id), _layer) in layers {
        if !realms.entries.contains_key(&RealmId(*realm_id)) {
            continue;
        }
        let Some(target) = targets.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Window {
            continue;
        }
        let Some(window_id) = target.window_id else {
            continue;
        };
        match map.get_mut(realm_id) {
            Some(existing_window_id) => {
                if window_id < *existing_window_id {
                    *existing_window_id = window_id;
                }
            }
            None => {
                map.insert(*realm_id, window_id);
            }
        }
    }
    map
}

fn collect_layers_by_target(
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
) -> HashMap<TargetId, Vec<u32>> {
    let mut by_target: HashMap<TargetId, Vec<u32>> = HashMap::new();
    for ((realm_id, target_id), _layer) in layers {
        by_target.entry(*target_id).or_default().push(*realm_id);
    }
    by_target
}

fn topo_with_soft_cuts(
    targets: &HashSet<TargetId>,
    edges: &[TargetEdge],
) -> (Vec<TargetId>, Vec<TargetEdge>) {
    let mut final_order = Vec::new();
    let mut cut_edges = Vec::new();
    let mut remaining_targets: HashSet<TargetId> = targets.iter().copied().collect();
    let mut remaining_edges: Vec<TargetEdge> = edges.to_vec();
    let mut guard = 0;

    while !remaining_targets.is_empty() {
        guard += 1;
        if guard > 64 {
            let mut leftover: Vec<_> = remaining_targets.iter().copied().collect();
            leftover.sort_by_key(|id| id.0);
            final_order.extend(leftover);
            break;
        }

        let order = topo_order(&remaining_targets, &remaining_edges);
        for node in &order {
            remaining_targets.remove(node);
        }
        final_order.extend(order);

        if remaining_targets.is_empty() {
            break;
        }

        let mut pruned = Vec::new();
        for edge in remaining_edges {
            if remaining_targets.contains(&edge.parent) && remaining_targets.contains(&edge.child) {
                cut_edges.push(edge);
            } else {
                pruned.push(edge);
            }
        }
        remaining_edges = pruned;
    }

    (final_order, cut_edges)
}

fn topo_order(targets: &HashSet<TargetId>, edges: &[TargetEdge]) -> Vec<TargetId> {
    let mut incoming: HashMap<TargetId, usize> = targets.iter().map(|id| (*id, 0)).collect();

    for edge in edges {
        if incoming.contains_key(&edge.child) {
            *incoming.entry(edge.child).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<TargetId> = incoming
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
        .collect();
    let mut queue_vec: Vec<_> = queue.drain(..).collect();
    queue_vec.sort_by_key(|id| id.0);
    let mut queue: VecDeque<TargetId> = queue_vec.into();

    let mut edges_by_parent: HashMap<TargetId, Vec<TargetId>> = HashMap::new();
    for edge in edges {
        edges_by_parent
            .entry(edge.parent)
            .or_default()
            .push(edge.child);
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(children) = edges_by_parent.get(&node) {
            for child in children {
                if let Some(count) = incoming.get_mut(child) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        queue.push_back(*child);
                    }
                }
            }
        }
    }

    order
}
