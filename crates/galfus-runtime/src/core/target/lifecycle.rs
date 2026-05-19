use std::collections::HashSet;

use crate::core::realm::{RealmId, SurfaceId, UniversalState};
use crate::core::target::{TargetId, resolve::remove_auto_link_for_layer};

pub fn prune_target_graph_cache(universal_state: &mut UniversalState) {
    universal_state
        .targets
        .target_graph_cache
        .prune_dead_entries(
            &universal_state.targets.targets.entries,
            &universal_state.targets.target_layers.entries,
            &universal_state.composition.realms,
        );
}

pub fn dispose_layer(
    universal_state: &mut UniversalState,
    realm_id: u32,
    target_id: TargetId,
) -> bool {
    if universal_state
        .targets
        .target_layers
        .entries
        .remove(&(realm_id, target_id))
        .is_none()
    {
        return false;
    }

    remove_auto_link_for_layer(universal_state, realm_id, target_id);
    let has_layer_for_target = universal_state
        .targets
        .target_layers
        .entries
        .keys()
        .any(|(_, layer_target)| *layer_target == target_id);
    if !has_layer_for_target {
        universal_state
            .interaction
            .input_routing
            .focus_targets
            .retain(|_, focus_target_id| *focus_target_id != target_id);
    }
    prune_target_graph_cache(universal_state);
    true
}

pub fn dispose_target(universal_state: &mut UniversalState, target_id: TargetId) -> bool {
    if universal_state
        .targets
        .targets
        .entries
        .remove(&target_id)
        .is_none()
    {
        return false;
    }

    universal_state
        .targets
        .target_layers
        .entries
        .retain(|(_, layer_target), _| *layer_target != target_id);

    let remove_keys: Vec<_> = universal_state
        .targets
        .auto_links
        .keys()
        .filter(|(_, layer_target)| *layer_target == target_id)
        .copied()
        .collect();
    for (realm_id, layer_target) in remove_keys {
        remove_auto_link_for_layer(universal_state, realm_id, layer_target);
    }

    universal_state
        .interaction
        .input_routing
        .focus_targets
        .retain(|_, focus_target_id| *focus_target_id != target_id);
    universal_state
        .interaction
        .ui
        .external_textures
        .remove(&(target_id.0 as u32));
    universal_state
        .interaction
        .ui
        .target_size_requests
        .remove(&target_id.0);
    universal_state
        .targets
        .target_ui_realm_index
        .remove(&target_id);
    universal_state
        .scene
        .render_resources
        .target_texture_binds
        .retain(|_, binding| binding.target_id != target_id);
    prune_target_graph_cache(universal_state);
    true
}

pub fn dispose_window_targets(
    universal_state: &mut UniversalState,
    window_id: u32,
) -> HashSet<TargetId> {
    let targets_to_remove: HashSet<_> = universal_state
        .targets
        .targets
        .entries
        .iter()
        .filter_map(|(target_id, target)| {
            (target.window_id == Some(window_id)).then_some(*target_id)
        })
        .collect();
    if targets_to_remove.is_empty() {
        return targets_to_remove;
    }

    let layers_to_remove: Vec<_> = universal_state
        .targets
        .target_layers
        .entries
        .keys()
        .filter(|(_, layer_target)| targets_to_remove.contains(layer_target))
        .copied()
        .collect();
    for (layer_realm_id, layer_target_id) in layers_to_remove {
        universal_state
            .targets
            .target_layers
            .entries
            .remove(&(layer_realm_id, layer_target_id));
        remove_auto_link_for_layer(universal_state, layer_realm_id, layer_target_id);
    }

    universal_state
        .targets
        .targets
        .entries
        .retain(|target_id, _| !targets_to_remove.contains(target_id));
    universal_state
        .scene
        .render_resources
        .target_texture_binds
        .retain(|_, binding| !targets_to_remove.contains(&binding.target_id));
    universal_state
        .interaction
        .ui
        .external_textures
        .retain(|target_id, _| !targets_to_remove.contains(&TargetId(*target_id as u64)));
    universal_state
        .interaction
        .ui
        .target_size_requests
        .retain(|target_id, _| !targets_to_remove.contains(&TargetId(*target_id)));
    universal_state
        .targets
        .target_ui_realm_index
        .retain(|target_id, _| !targets_to_remove.contains(target_id));
    universal_state
        .interaction
        .input_routing
        .focus_targets
        .retain(|_, target_id| !targets_to_remove.contains(target_id));
    prune_target_graph_cache(universal_state);

    targets_to_remove
}

pub fn dispose_realm_layers(universal_state: &mut UniversalState, realm_id: RealmId) {
    let removed_layers: Vec<_> = universal_state
        .targets
        .target_layers
        .entries
        .keys()
        .filter(|(layer_realm, _)| *layer_realm == realm_id.0)
        .copied()
        .collect();
    for (layer_realm, layer_target) in removed_layers {
        universal_state
            .targets
            .target_layers
            .entries
            .remove(&(layer_realm, layer_target));
        remove_auto_link_for_layer(universal_state, layer_realm, layer_target);
    }
    prune_target_graph_cache(universal_state);
}

pub fn dispose_surface_auto_links(
    universal_state: &mut UniversalState,
    realm_id: RealmId,
    surface_id: SurfaceId,
) {
    let keys: Vec<_> = universal_state
        .targets
        .auto_links
        .iter()
        .filter_map(|((layer_realm, layer_target), link)| {
            if *layer_realm == realm_id.0 || link.surface_id == surface_id {
                Some((*layer_realm, *layer_target))
            } else {
                None
            }
        })
        .collect();
    for (layer_realm, layer_target) in keys {
        remove_auto_link_for_layer(universal_state, layer_realm, layer_target);
    }
}
