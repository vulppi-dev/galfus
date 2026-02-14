use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::realm::{RealmId, RealmTable};
use crate::core::target::{TargetId, TargetLayerLayout, TargetLayerState, TargetState};

pub(super) fn hash_targets_layers_and_realms(
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

pub(super) fn hash_entries(
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
