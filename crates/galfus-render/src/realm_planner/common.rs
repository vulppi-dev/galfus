use std::collections::{HashMap, HashSet};
use std::hash::Hasher;

use galfus_realm_core::{
    AutoLink, ConnectorId, FrameCutEdge, RealmGraphPlan, RealmId, RealmState, SurfaceCache,
    SurfaceId, TargetId, TargetKind, TargetLayerState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentLayerBinding {
    pub target_id: TargetId,
    pub camera_id: Option<u32>,
    pub environment_id: Option<u32>,
    pub z_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmEnvironmentBindingPlan {
    pub realm_environment_id: Option<u32>,
    pub camera_environment_ids: HashMap<u32, u32>,
}

pub fn collect_cut_connectors(plan: &RealmGraphPlan) -> HashSet<ConnectorId> {
    plan.cut_edges
        .iter()
        .filter_map(|edge| edge.connector_id)
        .collect()
}

pub fn update_surface_cache(
    surface_cache: &mut SurfaceCache,
    connectors: &[(ConnectorId, SurfaceId)],
) {
    for (connector_id, source_surface) in connectors {
        surface_cache
            .last_good
            .insert(*connector_id, *source_surface);
        surface_cache
            .fallback
            .entry(*connector_id)
            .or_insert(*source_surface);
    }
}

pub fn collect_connectors_by_realm(
    connectors: &[(ConnectorId, RealmId)],
) -> HashMap<RealmId, Vec<ConnectorId>> {
    let mut map: HashMap<RealmId, Vec<ConnectorId>> = HashMap::new();
    for (connector_id, realm_id) in connectors {
        map.entry(*realm_id).or_default().push(*connector_id);
    }
    for connectors in map.values_mut() {
        connectors.sort_by_key(|id| id.0);
    }
    map
}

pub fn resolve_realm_surface(
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
    realm_id: RealmId,
) -> Option<SurfaceId> {
    realm_output_surfaces.get(&realm_id).copied().flatten()
}

pub fn map_realms_to_windows(
    existing_realms: &HashSet<RealmId>,
    layer_windows: &[(RealmId, u32)],
    presents: &[(SurfaceId, u32)],
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
) -> HashMap<RealmId, u32> {
    let mut map = HashMap::new();
    for (realm_id, window_id) in layer_windows {
        if !existing_realms.contains(realm_id) {
            continue;
        }
        match map.get_mut(realm_id) {
            Some(existing_window_id) => {
                if *window_id < *existing_window_id {
                    *existing_window_id = *window_id;
                }
            }
            None => {
                map.insert(*realm_id, *window_id);
            }
        }
    }

    let mut surface_to_realm = HashMap::new();
    for (realm_id, surface_id) in realm_output_surfaces {
        if let Some(surface_id) = surface_id {
            surface_to_realm.insert(*surface_id, *realm_id);
        }
    }
    for (surface_id, window_id) in presents {
        if let Some(realm_id) = surface_to_realm.get(surface_id) {
            map.entry(*realm_id).or_insert(*window_id);
        }
    }
    map
}

pub fn update_present_size_cache(
    presents: &[(SurfaceId, u32)],
    window_sizes: &HashMap<u32, glam::UVec2>,
    cache: &mut HashMap<SurfaceId, glam::UVec2>,
    cache_hash: &mut u64,
) -> bool {
    let mut chosen_windows: HashMap<SurfaceId, u32> = HashMap::new();
    for (surface_id, window_id) in presents {
        chosen_windows
            .entry(*surface_id)
            .and_modify(|current_window_id| {
                if *window_id < *current_window_id {
                    *current_window_id = *window_id;
                }
            })
            .or_insert(*window_id);
    }

    let mut aggregate_hash = 0_u64;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut changed = false;

    for (surface_id, window_id) in &chosen_windows {
        let size = window_sizes
            .get(window_id)
            .copied()
            .unwrap_or_else(|| glam::UVec2::new(0, 0));
        if cache.get(surface_id).copied() != Some(size) {
            cache.insert(*surface_id, size);
            changed = true;
        }
        hasher.write_u32(surface_id.0);
        hasher.write_u32(*window_id);
        hasher.write_u32(size.x);
        hasher.write_u32(size.y);
        aggregate_hash ^= hasher.finish();
        hasher = std::collections::hash_map::DefaultHasher::new();
    }

    let previous_len = cache.len();
    cache.retain(|surface_id, _| chosen_windows.contains_key(surface_id));
    if cache.len() != previous_len {
        changed = true;
    }

    if !changed && aggregate_hash == *cache_hash {
        return false;
    }

    *cache_hash = aggregate_hash;
    true
}

pub fn should_render_realm(realm_state: &mut RealmState, frame_index: u64) -> bool {
    let importance = realm_state.importance;
    if importance == 0 {
        return false;
    }
    let base_interval: u64 = match importance {
        1 => 1,
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let cache_multiplier: u64 = match realm_state.cache_policy {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 1,
    };
    let interval = base_interval.saturating_mul(cache_multiplier);
    let should_render = frame_index.saturating_sub(realm_state.last_render_frame) >= interval;
    if should_render {
        realm_state.last_render_frame = frame_index;
    }
    should_render
}

pub fn build_target_surface_map(
    targets: &HashMap<TargetId, (TargetKind, Option<glam::UVec2>)>,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
) -> HashMap<TargetId, SurfaceId> {
    let mut chosen: HashMap<TargetId, (u32, SurfaceId)> = HashMap::new();

    for ((realm_id, target_id), link) in auto_links {
        let Some((kind, _size)) = targets.get(target_id) else {
            continue;
        };
        if *kind != TargetKind::Texture {
            continue;
        }

        match chosen.get(target_id) {
            Some((current_realm, _)) if *current_realm <= *realm_id => {}
            _ => {
                chosen.insert(*target_id, (*realm_id, link.surface_id));
            }
        }
    }

    chosen
        .into_iter()
        .map(|(target_id, (_realm_id, surface_id))| (target_id, surface_id))
        .collect()
}

pub fn collect_window_camera_target_sizes(
    layers: &HashMap<(u32, TargetId), TargetLayerState>,
    targets: &HashMap<TargetId, (Option<u32>, Option<glam::UVec2>)>,
    realm_id: RealmId,
    window_id: u32,
    window_size: glam::UVec2,
) -> HashMap<u32, glam::UVec2> {
    const DEFAULT_CH_WIDTH: f32 = 8.0;
    let mut sizes = HashMap::new();
    for layer in layers.values() {
        if layer.realm_id != realm_id.0 {
            continue;
        }
        if layer.enabled_camera_ids.is_empty() {
            continue;
        }
        let Some((target_window_id, target_size)) = targets.get(&layer.target_id) else {
            continue;
        };
        if *target_window_id != Some(window_id) {
            continue;
        }

        let ref_width = window_size.x.max(1) as f32;
        let ref_height = window_size.y.max(1) as f32;
        let layout_width = layer
            .layout
            .width
            .resolve(ref_width, DEFAULT_CH_WIDTH)
            .max(1.0)
            .round() as u32;
        let layout_height = layer
            .layout
            .height
            .resolve(ref_height, DEFAULT_CH_WIDTH)
            .max(1.0)
            .round() as u32;

        let size = target_size.unwrap_or(glam::UVec2::new(layout_width, layout_height));
        let normalized_size = glam::UVec2::new(size.x.max(1), size.y.max(1));
        for camera_id in &layer.enabled_camera_ids {
            sizes.insert(*camera_id, normalized_size);
        }
    }
    sizes
}

pub fn plan_realm_environment_bindings(
    layers: &[EnvironmentLayerBinding],
) -> RealmEnvironmentBindingPlan {
    let mut ordered_layers = layers.to_vec();
    ordered_layers.sort_by_key(|layer| (layer.z_index, layer.target_id.0));

    let mut realm_environment_id = None;
    let mut camera_environment_ids = HashMap::new();
    for layer in ordered_layers {
        let Some(environment_id) = layer.environment_id else {
            continue;
        };
        if let Some(camera_id) = layer.camera_id {
            camera_environment_ids.insert(camera_id, environment_id);
        } else {
            realm_environment_id = Some(environment_id);
        }
    }

    RealmEnvironmentBindingPlan {
        realm_environment_id,
        camera_environment_ids,
    }
}

pub fn build_soft_cut_diagnostic(
    cut_edges: &[FrameCutEdge],
    previous_cut_edges: usize,
    frame_index: u64,
) -> Option<String> {
    if cut_edges.is_empty() || !(previous_cut_edges == 0 || previous_cut_edges != cut_edges.len()) {
        return None;
    }

    let connectors: Vec<_> = cut_edges
        .iter()
        .filter_map(|edge| edge.connector_id)
        .collect();
    let connector_text = if connectors.is_empty() {
        "none".to_string()
    } else {
        connectors
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };

    Some(format!(
        "frame={} cut_edges={} connectors={}",
        frame_index,
        cut_edges.len(),
        connector_text
    ))
}
