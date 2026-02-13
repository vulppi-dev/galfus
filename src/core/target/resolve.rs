use std::collections::{HashMap, HashSet};

use crate::core::realm::{
    AutoLink, ConnectorId, ConnectorState, PresentState, RealmId, SurfaceKind, SurfaceState,
    UniversalState,
};
use crate::core::state::EngineState;
use crate::core::target::{TargetLayerState, TargetId, TargetKind};

pub fn sync_auto_graph(engine_state: &mut EngineState) {
    let mut desired_binds: Vec<TargetLayerState> = engine_state
        .universal_state
        .target_layers
        .entries
        .values()
        .cloned()
        .collect();
    desired_binds.sort_by_key(|bind| (bind.realm_id, bind.target_id.0));
    let desired_keys: HashSet<(u32, TargetId)> = desired_binds
        .iter()
        .map(|bind| (bind.realm_id, bind.target_id))
        .collect();

    let existing_keys: Vec<_> = engine_state
        .universal_state
        .auto_links
        .keys()
        .copied()
        .collect();
    for key in existing_keys {
        if !desired_keys.contains(&key) {
            remove_auto_link(&mut engine_state.universal_state, key);
        }
    }

    let mut primary_targets: HashMap<u32, TargetId> = HashMap::new();

    for bind in desired_binds {
        let key = (bind.realm_id, bind.target_id);
        let target = match engine_state
            .universal_state
            .targets
            .entries
            .get(&bind.target_id)
        {
            Some(target) => target.clone(),
            None => {
                remove_auto_link(&mut engine_state.universal_state, key);
                continue;
            }
        };

        let primary_target = primary_targets
            .entry(bind.realm_id)
            .or_insert(bind.target_id);
        let is_primary = *primary_target == bind.target_id;

        let mut surface_id = engine_state
            .universal_state
            .realms
            .entries
            .get(&RealmId(bind.realm_id))
            .and_then(|entry| entry.value.output_surface);

        if surface_id.is_none() {
            let desired_surface = surface_state_for_target(engine_state, &target);
            surface_id = Some(engine_state.universal_state.surfaces.alloc(desired_surface));
            if let Some(entry) = engine_state
                .universal_state
                .realms
                .entries
                .get_mut(&RealmId(bind.realm_id))
            {
                entry.value.output_surface = surface_id;
            }
        } else if is_primary {
            let desired_surface = surface_state_for_target(engine_state, &target);
            if let Some(surface_id) = surface_id {
                if let Some(entry) = engine_state
                    .universal_state
                    .surfaces
                    .entries
                    .get_mut(&surface_id)
                {
                    if !surface_state_matches(&entry.value, &desired_surface) {
                        entry.value = desired_surface;
                    }
                }
            }
        }

        let Some(surface_id) = surface_id else {
            continue;
        };

        if let Some(link) = engine_state.universal_state.auto_links.get(&key).cloned() {
            let needs_rebuild =
                link.surface_id != surface_id
                    || link.present_id.is_none()
                        && matches!(target.kind, TargetKind::Window)
                    || link.connector_id.is_none()
                        && matches!(target.kind, TargetKind::RealmViewport | TargetKind::UiPlane);

            if needs_rebuild {
                remove_auto_link(&mut engine_state.universal_state, key);
            } else {
                if let Some(connector_id) = link.connector_id {
                    update_auto_link_layout(
                        &mut engine_state.universal_state,
                        Some(connector_id),
                        &bind,
                    );
                }
                continue;
            }
        }

        let mut connector_id = None;
        let mut present_id = None;

        match target.kind {
            TargetKind::Window => {
                if let Some(window_id) = target.window_id {
                    present_id = Some(engine_state.universal_state.presents.alloc(PresentState {
                        window_id,
                        surface: surface_id,
                    }));
                }
            }
            TargetKind::RealmViewport | TargetKind::UiPlane => {
                if let Some(window_id) = target.window_id {
                    if let Some(host_realm) =
                        find_host_realm_for_window(&engine_state.universal_state, window_id)
                    {
                        connector_id = Some(engine_state.universal_state.connectors.alloc(
                            ConnectorState {
                                target_realm: host_realm,
                                source_surface: surface_id,
                                rect: bind.layout.rect,
                                z_index: bind.layout.z_index,
                                blend_mode: bind.layout.blend_mode,
                                clip: bind.layout.clip,
                                input_flags: bind.layout.input_flags,
                            },
                        ));
                    }
                }
            }
            TargetKind::Texture => {}
        }

        engine_state.universal_state.auto_links.insert(
            key,
            AutoLink {
                surface_id,
                connector_id,
                present_id,
            },
        );
    }
}

pub(crate) fn remove_auto_link_for_layer(
    universal: &mut UniversalState,
    realm_id: u32,
    target_id: TargetId,
) {
    remove_auto_link(universal, (realm_id, target_id));
}

fn update_auto_link_layout(
    universal: &mut UniversalState,
    connector_id: Option<ConnectorId>,
    bind: &TargetLayerState,
) {
    let Some(connector_id) = connector_id else {
        return;
    };
    let Some(entry) = universal.connectors.get_mut(connector_id) else {
        return;
    };

    entry.value.rect = bind.layout.rect;
    entry.value.z_index = bind.layout.z_index;
    entry.value.blend_mode = bind.layout.blend_mode;
    entry.value.clip = bind.layout.clip;
    entry.value.input_flags = bind.layout.input_flags;
}

fn remove_auto_link(universal: &mut UniversalState, key: (u32, TargetId)) {
    let realm_id = key.0;
    let Some(link) = universal.auto_links.remove(&key) else {
        return;
    };

    if let Some(connector_id) = link.connector_id {
        universal.connectors.remove(connector_id);
        universal
            .input_routing
            .captures
            .retain(|_, capture| capture.connector_id != connector_id);
        universal.surface_cache.last_good.remove(&connector_id);
        universal.surface_cache.fallback.remove(&connector_id);
    }
    if let Some(present_id) = link.present_id {
        universal.presents.remove(present_id);
    }
    if let Some(entry) = universal.realms.entries.get_mut(&RealmId(realm_id)) {
        if entry.value.output_surface == Some(link.surface_id) {
            let surface_id = link.surface_id;
            let still_used = universal
                .auto_links
                .iter()
                .any(|((realm, _), link)| *realm == realm_id && link.surface_id == surface_id);
            if !still_used {
                entry.value.output_surface = None;
            }
        }
    }

    let surface_still_used = universal
        .auto_links
        .values()
        .any(|link_entry| link_entry.surface_id == link.surface_id);
    if !surface_still_used {
        universal.surfaces.remove(link.surface_id);
        universal
            .surface_cache
            .last_good
            .retain(|_, source| *source != link.surface_id);
        universal
            .surface_cache
            .fallback
            .retain(|_, source| *source != link.surface_id);
    }
}

fn find_host_realm_for_window(universal: &UniversalState, window_id: u32) -> Option<RealmId> {
    let mut chosen: Option<RealmId> = None;
    for (realm_id, entry) in universal.realms.entries.iter() {
        if entry.value.host_window_id != Some(window_id) {
            continue;
        }
        if let Some(current) = chosen {
            if realm_id.0 < current.0 {
                chosen = Some(*realm_id);
            }
        } else {
            chosen = Some(*realm_id);
        }
    }
    chosen
}

fn surface_state_for_target(
    engine_state: &EngineState,
    target: &crate::core::target::TargetState,
) -> SurfaceState {
    let size = match target.kind {
        TargetKind::Texture => target.size.unwrap_or_else(|| glam::UVec2::new(1, 1)),
        TargetKind::Window | TargetKind::RealmViewport | TargetKind::UiPlane => target
            .window_id
            .and_then(|window_id| engine_state.window.states.get(&window_id))
            .map(|state| state.inner_size)
            .unwrap_or_else(|| glam::UVec2::new(1, 1)),
    };

    SurfaceState {
        kind: match target.kind {
            TargetKind::Window => SurfaceKind::Onscreen,
            TargetKind::RealmViewport | TargetKind::UiPlane | TargetKind::Texture => {
                SurfaceKind::Offscreen
            }
        },
        size,
        format_policy: target.format_policy,
        alpha_policy: target.alpha_policy,
        msaa_samples: target.msaa_samples,
    }
}

fn surface_state_matches(a: &SurfaceState, b: &SurfaceState) -> bool {
    a.kind == b.kind
        && a.size == b.size
        && a.format_policy == b.format_policy
        && a.alpha_policy == b.alpha_policy
        && a.msaa_samples == b.msaa_samples
}
