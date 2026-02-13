use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::target::{
    TargetId, TargetKind, TargetLayerLayout, TargetLayerState, TargetState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetEdge {
    pub parent: TargetId,
    pub child: TargetId,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphPlan {
    pub edges: Vec<TargetEdge>,
    pub order: Vec<TargetId>,
    pub cut_edges: Vec<TargetEdge>,
}

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
        let mut parents: HashMap<TargetId, TargetId> = HashMap::new();

        for (target_id, target) in targets {
            match target.kind {
                TargetKind::Window | TargetKind::Texture => {}
                TargetKind::RealmViewport | TargetKind::UiPlane => {
                    if let Some(parent) =
                        infer_parent_from_layers(layers, realms, *target_id, &window_targets)
                    {
                        parents.insert(*target_id, parent);
                    }
                }
            }
        }

        let mut edges = Vec::new();
        for (child, parent) in parents {
            edges.push(TargetEdge { parent, child });
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
pub struct TargetGraphDiff {
    pub added_targets: Vec<TargetId>,
    pub removed_targets: Vec<TargetId>,
    pub updated_targets: Vec<TargetId>,
    pub added_layers: Vec<(u32, TargetId)>,
    pub removed_layers: Vec<(u32, TargetId)>,
    pub updated_layers: Vec<(u32, TargetId)>,
    pub dirty_targets: Vec<TargetId>,
    pub plan_dirty: bool,
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
    diff.removed_layers.sort_by_key(|(realm, id)| (*realm, id.0));
    diff.updated_layers.sort_by_key(|(realm, id)| (*realm, id.0));
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
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    realms: &RealmTable,
    target_id: TargetId,
    window_targets: &HashMap<u32, TargetId>,
) -> Option<TargetId> {
    let mut chosen_window: Option<u32> = None;
    let mut chosen_realm: Option<u32> = None;

    for layer in layers.values() {
        if layer.target_id != target_id {
            continue;
        }

        let realm_id = RealmId(layer.realm_id);
        let realm = realms.entries.get(&realm_id)?;
        let Some(host_window_id) = realm.value.host_window_id else {
            continue;
        };

        match chosen_window {
            None => {
                chosen_window = Some(host_window_id);
                chosen_realm = Some(layer.realm_id);
            }
            Some(current_window) => {
                if host_window_id < current_window {
                    chosen_window = Some(host_window_id);
                    chosen_realm = Some(layer.realm_id);
                } else if host_window_id == current_window {
                    let current_realm = chosen_realm.unwrap_or(u32::MAX);
                    if layer.realm_id < current_realm {
                        chosen_realm = Some(layer.realm_id);
                    }
                }
            }
        }
    }

    let window_id = chosen_window?;
    window_targets.get(&window_id).copied()
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
        if targets.contains(&edge.child) {
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
    for children in edges_by_parent.values_mut() {
        children.sort_by_key(|id| id.0);
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

fn hash_targets_layers_and_realms(
    targets: &HashMap<TargetId, TargetState>,
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    realms: &RealmTable,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    let mut target_hashes: Vec<_> = targets
        .iter()
        .map(|(id, state)| (*id, hash_target_state(state)))
        .collect();
    target_hashes.sort_by_key(|(id, _)| id.0);
    for (id, entry_hash) in target_hashes {
        id.hash(&mut hasher);
        entry_hash.hash(&mut hasher);
    }

    let mut layer_hashes: Vec<_> = layers
        .iter()
        .map(|((realm_id, target_id), layer)| ((*realm_id, *target_id), hash_layer_state(layer)))
        .collect();
    layer_hashes.sort_by_key(|((realm_id, target_id), _)| (*realm_id, target_id.0));
    for ((realm_id, target_id), entry_hash) in layer_hashes {
        realm_id.hash(&mut hasher);
        target_id.hash(&mut hasher);
        entry_hash.hash(&mut hasher);
    }

    let mut realm_hashes: Vec<_> = realms
        .entries
        .iter()
        .map(|(realm_id, entry)| (*realm_id, hash_realm_host(entry.value.host_window_id)))
        .collect();
    realm_hashes.sort_by_key(|(realm_id, _)| realm_id.0);
    for (realm_id, entry_hash) in realm_hashes {
        realm_id.hash(&mut hasher);
        entry_hash.hash(&mut hasher);
    }

    hasher.finish()
}

fn hash_entries(
    targets: &HashMap<TargetId, TargetState>,
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    realms: &RealmTable,
) -> (
    HashMap<TargetId, u64>,
    HashMap<(u32, TargetId), u64>,
    HashMap<RealmId, u64>,
) {
    let mut target_hashes = HashMap::new();
    for (id, state) in targets {
        target_hashes.insert(*id, hash_target_state(state));
    }

    let mut layer_hashes = HashMap::new();
    for (key, layer) in layers {
        layer_hashes.insert(*key, hash_layer_state(layer));
    }

    let mut realm_hashes = HashMap::new();
    for (realm_id, entry) in &realms.entries {
        realm_hashes.insert(*realm_id, hash_realm_host(entry.value.host_window_id));
    }

    (target_hashes, layer_hashes, realm_hashes)
}

fn hash_target_state(state: &TargetState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    state.kind.hash(&mut hasher);
    state.window_id.hash(&mut hasher);
    if let Some(size) = state.size {
        size.x.hash(&mut hasher);
        size.y.hash(&mut hasher);
    }
    hash_texture_format(state.format_policy, &mut hasher);
    hash_alpha_mode(state.alpha_policy, &mut hasher);
    state.msaa_samples.hash(&mut hasher);
    hasher.finish()
}

fn hash_layer_state(layer: &TargetLayerState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_layout(&layer.layout, &mut hasher);
    hasher.finish()
}

fn hash_realm_host(window_id: Option<u32>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    window_id.hash(&mut hasher);
    hasher.finish()
}

fn hash_layout(layout: &TargetLayerLayout, hasher: &mut impl Hasher) {
    hash_f32(layout.rect.x, hasher);
    hash_f32(layout.rect.y, hasher);
    hash_f32(layout.rect.z, hasher);
    hash_f32(layout.rect.w, hasher);
    layout.z_index.hash(hasher);
    layout.blend_mode.hash(hasher);
    if let Some(clip) = layout.clip {
        hash_f32(clip.x, hasher);
        hash_f32(clip.y, hasher);
        hash_f32(clip.z, hasher);
        hash_f32(clip.w, hasher);
    }
    layout.input_flags.hash(hasher);
}

fn hash_f32(value: f32, hasher: &mut impl Hasher) {
    value.to_bits().hash(hasher);
}

fn hash_texture_format(value: Option<wgpu::TextureFormat>, hasher: &mut impl Hasher) {
    if let Some(format) = value {
        std::mem::discriminant(&format).hash(hasher);
    }
}

fn hash_alpha_mode(value: Option<wgpu::CompositeAlphaMode>, hasher: &mut impl Hasher) {
    if let Some(mode) = value {
        std::mem::discriminant(&mode).hash(hasher);
    }
}
