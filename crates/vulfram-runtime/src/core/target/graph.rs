use std::collections::{HashMap, HashSet};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::render::SceneRuntimeState;
use crate::core::resources::{
    PBR_INVALID_SLOT, STANDARD_INVALID_SLOT, TargetTextureBinding,
};
use crate::core::target::graph_hash::{hash_entries, hash_targets_layers_and_realms};
use crate::core::target::{TargetId, TargetLayerState, TargetState};
#[allow(unused_imports)]
pub use vulfram_realm_core::{RenderInvocation, TargetEdge, TargetGraphDiff, TargetGraphPlan};

#[derive(Debug, Default)]
pub struct TargetGraphPlanner;

impl TargetGraphPlanner {
    pub fn build_plan(
        &self,
        targets: &HashMap<TargetId, TargetState>,
        target_dependencies: &[TargetEdge],
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &RealmTable,
    ) -> TargetGraphPlan {
        let target_semantics = targets
            .iter()
            .map(|(target_id, target)| (*target_id, (target.kind, target.window_id)))
            .collect();
        let realm_ids = realms.entries.keys().copied().collect();
        vulfram_realm_core::TargetGraphPlanner.build_plan(
            &target_semantics,
            target_dependencies,
            layers,
            &realm_ids,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphCache {
    pub last_hash: u64,
    pub last_dependency_hash: u64,
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
        target_dependencies: &[TargetEdge],
        layers: &HashMap<(u32, TargetId), TargetLayerState>,
        realms: &RealmTable,
    ) -> Option<&TargetGraphDiff> {
        let dependency_hash = hash_target_dependencies(target_dependencies);
        let dependency_changed = dependency_hash != self.last_dependency_hash;
        let current_hash = hash_targets_layers_and_realms(targets, layers, realms) ^ dependency_hash;
        if current_hash == self.last_hash {
            self.last_diff = None;
            return None;
        }

        let (target_hashes, layer_hashes, realm_hashes) = hash_entries(targets, layers, realms);
        let mut diff = diff_targets_layers_and_realms(
            &self.last_target_hashes,
            &self.last_layer_hashes,
            &self.last_realm_hashes,
            &target_hashes,
            &layer_hashes,
            &realm_hashes,
            layers,
        );
        if dependency_changed {
            diff.plan_dirty = true;
            let mut dependency_targets: Vec<_> = target_dependencies
                .iter()
                .flat_map(|edge| [edge.parent, edge.child])
                .collect();
            dependency_targets.sort_by_key(|id| id.0);
            dependency_targets.dedup();
            for target_id in dependency_targets {
                if !diff.dirty_targets.contains(&target_id) {
                    diff.dirty_targets.push(target_id);
                }
            }
            diff.dirty_targets.sort_by_key(|id| id.0);
        }
        self.last_target_hashes = target_hashes;
        self.last_layer_hashes = layer_hashes;
        self.last_realm_hashes = realm_hashes;
        self.last_hash = current_hash;
        self.last_dependency_hash = dependency_hash;
        if diff.plan_dirty {
            self.last_plan = TargetGraphPlanner.build_plan(targets, target_dependencies, layers, realms);
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

fn hash_target_dependencies(edges: &[TargetEdge]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut normalized = edges.to_vec();
    normalized.sort_by_key(|edge| (edge.parent.0, edge.child.0));
    normalized.dedup();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for edge in normalized {
        edge.parent.hash(&mut hasher);
        edge.child.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn collect_target_dependencies(
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    scene: &SceneRuntimeState,
) -> Vec<TargetEdge> {
    let mut textures_by_realm: HashMap<u32, HashSet<u32>> = HashMap::new();
    for (realm_id, entities) in &scene.realm3d.entities {
        let mut used_textures: HashSet<u32> = HashSet::new();
        for model in entities.models.values() {
            let Some(material_id) = model.material_id else {
                continue;
            };
            if let Some(material) = scene.realm3d.materials_standard.get(&material_id) {
                for texture_id in material.texture_ids {
                    if texture_id != STANDARD_INVALID_SLOT {
                        used_textures.insert(texture_id);
                    }
                }
                continue;
            }
            if let Some(material) = scene.realm3d.materials_pbr.get(&material_id) {
                for texture_id in material.texture_ids {
                    if texture_id != PBR_INVALID_SLOT {
                        used_textures.insert(texture_id);
                    }
                }
            }
        }
        if !used_textures.is_empty() {
            textures_by_realm.insert(realm_id.0, used_textures);
        }
    }

    let mut targets_by_realm: HashMap<u32, Vec<TargetId>> = HashMap::new();
    for ((realm_id, target_id), _layer) in layers {
        targets_by_realm.entry(*realm_id).or_default().push(*target_id);
    }
    for targets in targets_by_realm.values_mut() {
        targets.sort_by_key(|id| id.0);
        targets.dedup();
    }

    build_target_dependency_edges(&scene.render_resources.target_texture_binds, &textures_by_realm, &targets_by_realm)
}

fn build_target_dependency_edges(
    target_texture_binds: &HashMap<u32, TargetTextureBinding>,
    textures_by_realm: &HashMap<u32, HashSet<u32>>,
    targets_by_realm: &HashMap<u32, Vec<TargetId>>,
) -> Vec<TargetEdge> {
    let mut edges: Vec<TargetEdge> = Vec::new();
    for (texture_id, binding) in target_texture_binds {
        for (realm_id, textures) in textures_by_realm {
            if !textures.contains(texture_id) {
                continue;
            }
            let Some(targets) = targets_by_realm.get(realm_id) else {
                continue;
            };
            for target_id in targets {
                if *target_id == binding.target_id {
                    continue;
                }
                edges.push(TargetEdge {
                    parent: binding.target_id,
                    child: *target_id,
                });
            }
        }
    }
    edges.sort_by_key(|edge| (edge.parent.0, edge.child.0));
    edges.dedup();
    edges
}

pub fn collect_render_invocations(
    target_order: &[TargetId],
    targets: &HashMap<TargetId, TargetState>,
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    window_sizes: &HashMap<u32, glam::UVec2>,
    frame_id: u64,
) -> Vec<RenderInvocation> {
    let mut layers_by_target: HashMap<TargetId, Vec<&TargetLayerState>> = HashMap::new();
    for layer in layers.values() {
        if !layer.layout.enabled || layer.layout.opacity <= 0.0 {
            continue;
        }
        layers_by_target.entry(layer.target_id).or_default().push(layer);
    }
    for target_layers in layers_by_target.values_mut() {
        target_layers.sort_by_key(|layer| (layer.layout.z_index, layer.realm_id, layer.target_id.0));
    }

    let mut invocations = Vec::new();
    for target_id in target_order {
        let Some(target) = targets.get(target_id) else {
            continue;
        };
        let Some(target_layers) = layers_by_target.get(target_id) else {
            continue;
        };
        let target_size = resolve_target_size(target, window_sizes);
        for layer in target_layers {
            let resolved = vulfram_render::resolve_auto_graph_layout(target_size, &layer.layout);
            let x = resolved.rect.x.max(0.0).round() as u32;
            let y = resolved.rect.y.max(0.0).round() as u32;
            let w = resolved.rect.z.max(1.0).round() as u32;
            let h = resolved.rect.w.max(1.0).round() as u32;
            invocations.push(RenderInvocation {
                realm_id: layer.realm_id,
                target_id: *target_id,
                layer_key: (layer.realm_id, *target_id),
                resolved_rect_px: glam::UVec4::new(x, y, w, h),
                render_size_px: glam::UVec2::new(w, h),
                frame_id,
            });
        }
    }

    invocations
}

fn resolve_target_size(
    target: &TargetState,
    window_sizes: &HashMap<u32, glam::UVec2>,
) -> glam::UVec2 {
    match target.kind {
        crate::core::target::TargetKind::Window => target
            .window_id
            .and_then(|window_id| window_sizes.get(&window_id).copied())
            .unwrap_or_else(|| glam::UVec2::new(1, 1)),
        crate::core::target::TargetKind::Texture => target.size.unwrap_or_else(|| glam::UVec2::new(1, 1)),
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
