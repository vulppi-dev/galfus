use glam::UVec2;

use crate::core::realm::{RealmId, UniversalState};

pub(super) fn rebuild_input_routing_cache(universal: &mut UniversalState) {
    let snapshot = build_input_routing_topology_snapshot(universal);
    let topology_hash = vulfram_input::compute_input_topology_hash(&snapshot);
    if universal.input_routing.cache.topology_hash == topology_hash {
        return;
    }
    universal.input_routing.cache = vulfram_input::build_input_routing_cache(&snapshot);
}

fn build_input_routing_topology_snapshot(
    universal: &UniversalState,
) -> vulfram_input::InputRoutingTopologySnapshot {
    let realms = universal
        .realms
        .entries
        .iter()
        .map(|(realm_id, entry)| vulfram_input::InputRoutingRealmOutput {
            realm_id: *realm_id,
            output_surface: entry.value.output_surface,
        })
        .collect();

    let presents = universal
        .presents
        .entries
        .values()
        .map(|entry| vulfram_input::InputRoutingPresentBinding {
            window_id: entry.value.window_id,
            surface_id: entry.value.surface,
        })
        .collect();

    let target_order = universal
        .target_graph_cache
        .last_plan
        .order
        .iter()
        .enumerate()
        .map(|(index, target_id)| vulfram_input::InputRoutingTargetRank {
            target_id: *target_id,
            rank: index as i32,
        })
        .collect();

    let auto_links = universal
        .auto_links
        .iter()
        .filter_map(|((_, target_id), link)| {
            link.connector_id
                .map(|connector_id| vulfram_input::InputRoutingAutoLinkRecord {
                    target_id: *target_id,
                    connector_id,
                })
        })
        .collect();

    let layer_cameras = universal
        .target_layers
        .entries
        .iter()
        .map(
            |((realm_id, target_id), layer)| vulfram_input::InputRoutingLayerCameraRecord {
                realm_id: *realm_id,
                target_id: *target_id,
                camera_id: layer.camera_id,
            },
        )
        .collect();

    let connectors = universal
        .connectors
        .entries
        .iter()
        .filter_map(|(connector_id, entry)| {
            universal
                .surfaces
                .entries
                .get(&entry.value.source_surface)
                .map(|surface| vulfram_input::InputRoutingConnectorRecord {
                    connector_id: *connector_id,
                    state: entry.value.clone(),
                    source_size: surface.value.size,
                })
        })
        .collect();

    let surfaces = universal
        .surfaces
        .entries
        .iter()
        .map(
            |(surface_id, entry)| vulfram_input::InputRoutingSurfaceSizeRecord {
                surface_id: *surface_id,
                size: entry.value.size,
            },
        )
        .collect();

    vulfram_input::InputRoutingTopologySnapshot {
        realms,
        presents,
        target_order,
        auto_links,
        layer_cameras,
        connectors,
        surfaces,
    }
}

pub(super) use vulfram_input::{
    resolve_captured_connector, resolve_connector_for_target, resolve_focus_target,
};

pub(super) fn realm_surface_size(universal: &UniversalState, realm_id: RealmId) -> Option<UVec2> {
    let realm = universal.realms.entries.get(&realm_id)?;
    let surface_id = realm.value.output_surface?;
    let surface = universal.surfaces.entries.get(&surface_id)?;
    Some(surface.value.size)
}
