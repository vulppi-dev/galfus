use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::target::{TargetBindLayout, TargetBindState, TargetId, TargetKind, TargetState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetEdge {
    pub parent: TargetId,
    pub child: TargetId,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphPlan {
    pub edges: Vec<TargetEdge>,
    pub order: Vec<TargetId>,
}

#[derive(Debug, Default)]
pub struct TargetGraphPlanner;

impl TargetGraphPlanner {
    pub fn build_plan(
        &self,
        targets: &HashMap<TargetId, TargetState>,
        binds: &HashMap<(u32, TargetId), TargetBindState>,
        realms: &RealmTable,
    ) -> TargetGraphPlan {
        let window_targets = collect_window_targets(targets);
        let mut parents: HashMap<TargetId, TargetId> = HashMap::new();

        for (target_id, target) in targets.iter() {
            match target.kind {
                TargetKind::Window | TargetKind::Texture => {}
                TargetKind::RealmViewport | TargetKind::PanelEmbed => {
                    if let Some(parent) =
                        infer_parent_from_binds(binds, realms, *target_id, &window_targets)
                    {
                        parents.insert(*target_id, parent);
                    }
                }
            }
        }

        let mut edges = Vec::new();
        for (child, parent) in parents.iter() {
            edges.push(TargetEdge {
                parent: *parent,
                child: *child,
            });
        }

        let order = topo_sort_targets(targets, &edges);

        TargetGraphPlan { edges, order }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphDiff {
    pub added_targets: Vec<TargetId>,
    pub removed_targets: Vec<TargetId>,
    pub updated_targets: Vec<TargetId>,
    pub added_binds: Vec<(u32, TargetId)>,
    pub removed_binds: Vec<(u32, TargetId)>,
    pub updated_binds: Vec<(u32, TargetId)>,
    pub dirty_targets: Vec<TargetId>,
    pub plan_dirty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphCache {
    pub last_hash: u64,
    pub last_target_hashes: HashMap<TargetId, u64>,
    pub last_bind_hashes: HashMap<(u32, TargetId), u64>,
    pub last_plan: TargetGraphPlan,
    pub last_diff: Option<TargetGraphDiff>,
}

impl TargetGraphCache {
    pub fn update(
        &mut self,
        targets: &HashMap<TargetId, TargetState>,
        binds: &HashMap<(u32, TargetId), TargetBindState>,
        realms: &RealmTable,
    ) -> Option<&TargetGraphDiff> {
        let current_hash = hash_targets_and_binds(targets, binds);
        if current_hash == self.last_hash {
            self.last_diff = None;
            return None;
        }

        let (target_hashes, bind_hashes) = hash_entries(targets, binds);
        let diff = diff_targets_and_binds(
            &self.last_target_hashes,
            &self.last_bind_hashes,
            &target_hashes,
            &bind_hashes,
        );
        self.last_target_hashes = target_hashes;
        self.last_bind_hashes = bind_hashes;
        self.last_hash = current_hash;
        if diff.plan_dirty {
            self.last_plan = TargetGraphPlanner::default().build_plan(targets, binds, realms);
        }
        self.last_diff = Some(diff);
        self.last_diff.as_ref()
    }
}

fn diff_targets_and_binds(
    previous_targets: &HashMap<TargetId, u64>,
    previous_binds: &HashMap<(u32, TargetId), u64>,
    targets: &HashMap<TargetId, u64>,
    binds: &HashMap<(u32, TargetId), u64>,
) -> TargetGraphDiff {
    let mut diff = TargetGraphDiff::default();

    for (target_id, state) in targets.iter() {
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

    for (bind_key, state) in binds.iter() {
        match previous_binds.get(bind_key) {
            None => diff.added_binds.push(*bind_key),
            Some(prev) if prev != state => diff.updated_binds.push(*bind_key),
            _ => {}
        }
    }
    for bind_key in previous_binds.keys() {
        if !binds.contains_key(bind_key) {
            diff.removed_binds.push(*bind_key);
        }
    }

    diff.added_targets.sort_by_key(|id| id.0);
    diff.removed_targets.sort_by_key(|id| id.0);
    diff.updated_targets.sort_by_key(|id| id.0);
    diff.added_binds.sort_by_key(|(realm, id)| (*realm, id.0));
    diff.removed_binds.sort_by_key(|(realm, id)| (*realm, id.0));
    diff.updated_binds.sort_by_key(|(realm, id)| (*realm, id.0));

    let mut dirty_targets: HashSet<TargetId> = HashSet::new();
    for target_id in diff.added_targets.iter() {
        dirty_targets.insert(*target_id);
    }
    for target_id in diff.removed_targets.iter() {
        dirty_targets.insert(*target_id);
    }
    for target_id in diff.updated_targets.iter() {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in diff.added_binds.iter() {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in diff.removed_binds.iter() {
        dirty_targets.insert(*target_id);
    }
    for (_, target_id) in diff.updated_binds.iter() {
        dirty_targets.insert(*target_id);
    }
    diff.dirty_targets = dirty_targets.into_iter().collect();
    diff.dirty_targets.sort_by_key(|id| id.0);
    diff.plan_dirty = !diff.added_targets.is_empty()
        || !diff.removed_targets.is_empty()
        || !diff.updated_targets.is_empty()
        || !diff.added_binds.is_empty()
        || !diff.removed_binds.is_empty();

    diff
}

fn collect_window_targets(targets: &HashMap<TargetId, TargetState>) -> HashMap<u32, TargetId> {
    let mut map: HashMap<u32, TargetId> = HashMap::new();
    for (target_id, state) in targets.iter() {
        if state.kind != TargetKind::Window {
            continue;
        }
        if let Some(window_id) = state.owner_window_id {
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

fn infer_parent_from_binds(
    binds: &HashMap<(u32, TargetId), TargetBindState>,
    realms: &RealmTable,
    target_id: TargetId,
    window_targets: &HashMap<u32, TargetId>,
) -> Option<TargetId> {
    let mut chosen_window: Option<u32> = None;
    let mut chosen_realm: Option<u32> = None;

    for bind in binds.values() {
        if bind.target_id != target_id {
            continue;
        }

        let realm_id = RealmId(bind.realm_id);
        let realm = realms.entries.get(&realm_id)?;
        let Some(host_window_id) = realm.value.host_window_id else {
            continue;
        };

        match chosen_window {
            None => {
                chosen_window = Some(host_window_id);
                chosen_realm = Some(bind.realm_id);
            }
            Some(current_window) => {
                if host_window_id < current_window {
                    chosen_window = Some(host_window_id);
                    chosen_realm = Some(bind.realm_id);
                } else if host_window_id == current_window {
                    let current_realm = chosen_realm.unwrap_or(u32::MAX);
                    if bind.realm_id < current_realm {
                        chosen_realm = Some(bind.realm_id);
                    }
                }
            }
        }
    }

    let Some(window_id) = chosen_window else {
        return None;
    };
    window_targets.get(&window_id).copied()
}

fn topo_sort_targets(
    targets: &HashMap<TargetId, TargetState>,
    edges: &[TargetEdge],
) -> Vec<TargetId> {
    let mut incoming: HashMap<TargetId, usize> = targets.keys().map(|id| (*id, 0)).collect();

    for edge in edges {
        if let Some(count) = incoming.get_mut(&edge.child) {
            *count = count.saturating_add(1);
        }
    }

    let mut queue: std::collections::VecDeque<TargetId> = incoming
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
        .collect();
    let mut queue_vec: Vec<_> = queue.drain(..).collect();
    queue_vec.sort_by_key(|id| id.0);
    queue = queue_vec.into_iter().collect();

    let mut order = Vec::new();
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

    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(children) = edges_by_parent.get_mut(&node) {
            for child in children.iter() {
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

fn hash_targets_and_binds(
    targets: &HashMap<TargetId, TargetState>,
    binds: &HashMap<(u32, TargetId), TargetBindState>,
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

    let mut bind_hashes: Vec<_> = binds
        .iter()
        .map(|((realm_id, target_id), bind)| ((*realm_id, *target_id), hash_bind_state(bind)))
        .collect();
    bind_hashes.sort_by_key(|((realm_id, target_id), _)| (*realm_id, target_id.0));
    for ((realm_id, target_id), entry_hash) in bind_hashes {
        realm_id.hash(&mut hasher);
        target_id.hash(&mut hasher);
        entry_hash.hash(&mut hasher);
    }

    hasher.finish()
}

fn hash_entries(
    targets: &HashMap<TargetId, TargetState>,
    binds: &HashMap<(u32, TargetId), TargetBindState>,
) -> (HashMap<TargetId, u64>, HashMap<(u32, TargetId), u64>) {
    let mut target_hashes = HashMap::new();
    for (id, state) in targets.iter() {
        target_hashes.insert(*id, hash_target_state(state));
    }

    let mut bind_hashes = HashMap::new();
    for (key, bind) in binds.iter() {
        bind_hashes.insert(*key, hash_bind_state(bind));
    }

    (target_hashes, bind_hashes)
}

fn hash_target_state(state: &TargetState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    state.kind.hash(&mut hasher);
    state.owner_window_id.hash(&mut hasher);
    if let Some(size) = state.size_override {
        size.x.hash(&mut hasher);
        size.y.hash(&mut hasher);
    }
    hash_texture_format(state.format_policy, &mut hasher);
    hash_alpha_mode(state.alpha_policy, &mut hasher);
    state.msaa_samples.hash(&mut hasher);
    hasher.finish()
}

fn hash_bind_state(bind: &TargetBindState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_layout(&bind.layout, &mut hasher);
    hasher.finish()
}

fn hash_layout(layout: &TargetBindLayout, hasher: &mut impl Hasher) {
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
