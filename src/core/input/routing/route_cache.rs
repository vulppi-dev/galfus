use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use glam::UVec2;

use crate::core::realm::{
    ConnectorId, InputCapture, InputRoutingConnectorHit, RealmId, UniversalState,
};
use crate::core::target::TargetId;

pub(super) fn rebuild_input_routing_cache(universal: &mut UniversalState) {
    let topology_hash = compute_input_topology_hash(universal);
    if universal.input_routing.cache.topology_hash == topology_hash {
        return;
    }

    let mut realm_by_surface = HashMap::new();
    for (realm_id, entry) in universal.realms.entries.iter() {
        if let Some(surface_id) = entry.value.output_surface {
            realm_by_surface.insert(surface_id, *realm_id);
        }
    }

    let mut realm_by_window = HashMap::new();
    for present in universal.presents.entries.values() {
        if let Some(realm_id) = realm_by_surface.get(&present.value.surface) {
            realm_by_window.insert(present.value.window_id, (*realm_id, present.value.surface));
        }
    }

    let mut target_rank: HashMap<TargetId, i32> = HashMap::new();
    for (index, target_id) in universal
        .target_graph_cache
        .last_plan
        .order
        .iter()
        .enumerate()
    {
        target_rank.insert(*target_id, index as i32);
    }

    let mut connector_targets: HashMap<ConnectorId, TargetId> = HashMap::new();
    for ((_, target_id), link) in universal.auto_links.iter() {
        if let Some(connector_id) = link.connector_id {
            connector_targets.insert(connector_id, *target_id);
        }
    }

    let mut layer_camera_by_key: HashMap<(u32, TargetId), Option<u32>> = HashMap::new();
    for ((layer_realm_id, layer_target_id), layer) in universal.target_layers.entries.iter() {
        layer_camera_by_key.insert((*layer_realm_id, *layer_target_id), layer.camera_id);
    }

    let mut connectors_by_realm: HashMap<RealmId, Vec<InputRoutingConnectorHit>> = HashMap::new();
    for (connector_id, entry) in universal.connectors.entries.iter() {
        let Some(source_size) = universal
            .surfaces
            .entries
            .get(&entry.value.source_surface)
            .map(|surface| surface.value.size)
        else {
            continue;
        };
        let target_id = connector_targets.get(connector_id).copied();
        let rank = target_id
            .and_then(|id| target_rank.get(&id).copied())
            .unwrap_or(-1);
        connectors_by_realm
            .entry(entry.value.target_realm)
            .or_default()
            .push(InputRoutingConnectorHit {
                id: *connector_id,
                state: entry.value.clone(),
                source_size,
                target_id,
                target_rank: rank,
            });
    }

    for connectors in connectors_by_realm.values_mut() {
        connectors.sort_by(|a, b| {
            let z_cmp = b.state.z_index.cmp(&a.state.z_index);
            if z_cmp == Ordering::Equal {
                let rank_cmp = b.target_rank.cmp(&a.target_rank);
                if rank_cmp == Ordering::Equal {
                    b.id.0.cmp(&a.id.0)
                } else {
                    rank_cmp
                }
            } else {
                z_cmp
            }
        });
    }

    let cache = &mut universal.input_routing.cache;
    cache.topology_hash = topology_hash;
    cache.realm_by_surface = realm_by_surface;
    cache.realm_by_window = realm_by_window;
    cache.connector_targets = connector_targets;
    cache.layer_camera_by_key = layer_camera_by_key;
    cache.connectors_by_realm = connectors_by_realm;
}

fn compute_input_topology_hash(universal: &UniversalState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    universal.realms.entries.len().hash(&mut hasher);
    for (realm_id, entry) in universal.realms.entries.iter() {
        realm_id.hash(&mut hasher);
        entry.value.output_surface.hash(&mut hasher);
    }
    universal.presents.entries.len().hash(&mut hasher);
    for (present_id, entry) in universal.presents.entries.iter() {
        present_id.hash(&mut hasher);
        entry.value.window_id.hash(&mut hasher);
        entry.value.surface.hash(&mut hasher);
    }
    universal
        .target_graph_cache
        .last_plan
        .order
        .hash(&mut hasher);
    for ((realm_id, target_id), link) in universal.auto_links.iter() {
        realm_id.hash(&mut hasher);
        target_id.hash(&mut hasher);
        link.connector_id.hash(&mut hasher);
    }
    for ((realm_id, target_id), layer) in universal.target_layers.entries.iter() {
        realm_id.hash(&mut hasher);
        target_id.hash(&mut hasher);
        layer.camera_id.hash(&mut hasher);
    }
    for (connector_id, entry) in universal.connectors.entries.iter() {
        connector_id.hash(&mut hasher);
        entry.value.target_realm.hash(&mut hasher);
        entry.value.source_surface.hash(&mut hasher);
        entry.value.z_index.hash(&mut hasher);
        entry.value.blend_mode.hash(&mut hasher);
        entry.value.input_flags.hash(&mut hasher);
        entry.value.rect.x.to_bits().hash(&mut hasher);
        entry.value.rect.y.to_bits().hash(&mut hasher);
        entry.value.rect.z.to_bits().hash(&mut hasher);
        entry.value.rect.w.to_bits().hash(&mut hasher);
        if let Some(clip) = entry.value.clip {
            clip.x.to_bits().hash(&mut hasher);
            clip.y.to_bits().hash(&mut hasher);
            clip.z.to_bits().hash(&mut hasher);
            clip.w.to_bits().hash(&mut hasher);
        }
    }
    for (surface_id, entry) in universal.surfaces.entries.iter() {
        surface_id.hash(&mut hasher);
        entry.value.size.x.hash(&mut hasher);
        entry.value.size.y.hash(&mut hasher);
    }
    hasher.finish()
}

pub(super) fn resolve_captured_connector(
    captures: &HashMap<(u32, u64), InputCapture>,
    window_id: u32,
    pointer_id: u64,
) -> Option<InputCapture> {
    captures.get(&(window_id, pointer_id)).copied()
}

pub(super) fn resolve_focus_target(
    focus_targets: &HashMap<u32, TargetId>,
    window_id: u32,
) -> Option<TargetId> {
    focus_targets.get(&window_id).copied()
}

pub(super) fn resolve_connector_for_target(
    connectors: Option<&Vec<InputRoutingConnectorHit>>,
    target_id: TargetId,
) -> Option<ConnectorId> {
    let connectors = connectors?;
    for connector in connectors {
        if connector.target_id == Some(target_id) {
            return Some(connector.id);
        }
    }
    None
}

pub(super) fn realm_surface_size(universal: &UniversalState, realm_id: RealmId) -> Option<UVec2> {
    let realm = universal.realms.entries.get(&realm_id)?;
    let surface_id = realm.value.output_surface?;
    let surface = universal.surfaces.entries.get(&surface_id)?;
    Some(surface.value.size)
}
