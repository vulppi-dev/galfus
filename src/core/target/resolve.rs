use std::collections::{HashMap, HashSet};

use crate::core::realm::{
    AutoLink, ConnectorId, ConnectorState, PresentState, RealmId, SurfaceKind, SurfaceState,
    UniversalState,
};
use crate::core::state::EngineState;
use crate::core::target::{TargetBindState, TargetId, TargetKind};

pub fn sync_auto_graph(engine_state: &mut EngineState) {
    let desired_binds = select_primary_binds(&engine_state.universal_state.target_binds.entries);
    let mut desired_keys: HashSet<(u32, TargetId)> = desired_binds
        .values()
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

    for bind in desired_binds.values() {
        let key = (bind.realm_id, bind.target_id);
        if let Some(link) = engine_state.universal_state.auto_links.get(&key).cloned() {
            if let Some(target) = engine_state
                .universal_state
                .targets
                .entries
                .get(&bind.target_id)
            {
                let desired_surface = surface_state_for_target(engine_state, target);
                let needs_rebuild = engine_state
                    .universal_state
                    .surfaces
                    .entries
                    .get(&link.surface_id)
                    .map(|entry| !surface_state_matches(&entry.value, &desired_surface))
                    .unwrap_or(true)
                    || link.present_id.is_none() && matches!(target.kind, TargetKind::Window)
                    || link.connector_id.is_none()
                        && matches!(
                            target.kind,
                            TargetKind::ViewportEmbed | TargetKind::PanelEmbed
                        );

                if needs_rebuild {
                    remove_auto_link(&mut engine_state.universal_state, key);
                } else {
                    if let Some(connector_id) = link.connector_id {
                        update_auto_link_layout(
                            &mut engine_state.universal_state,
                            Some(connector_id),
                            bind,
                        );
                    }
                    continue;
                }
            } else {
                remove_auto_link(&mut engine_state.universal_state, key);
            }
        }

        let Some(target) = engine_state
            .universal_state
            .targets
            .entries
            .get(&bind.target_id)
        else {
            continue;
        };

        let desired_surface = surface_state_for_target(engine_state, target);
        let surface_id = engine_state.universal_state.surfaces.alloc(desired_surface);

        if let Some(entry) = engine_state
            .universal_state
            .realms
            .entries
            .get_mut(&RealmId(bind.realm_id))
        {
            entry.value.output_surface = Some(surface_id);
        }

        let mut connector_id = None;
        let mut present_id = None;

        match target.kind {
            TargetKind::Window => {
                if let Some(window_id) = target.owner_window_id {
                    present_id = Some(engine_state.universal_state.presents.alloc(PresentState {
                        window_id,
                        surface: surface_id,
                    }));
                }
            }
            TargetKind::ViewportEmbed | TargetKind::PanelEmbed => {
                if let Some(window_id) = target.owner_window_id {
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
        desired_keys.remove(&key);
    }
}

pub(crate) fn remove_auto_link_for_bind(
    universal: &mut UniversalState,
    realm_id: u32,
    target_id: TargetId,
) {
    remove_auto_link(universal, (realm_id, target_id));
}

fn update_auto_link_layout(
    universal: &mut UniversalState,
    connector_id: Option<ConnectorId>,
    bind: &TargetBindState,
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
            entry.value.output_surface = None;
        }
    }
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

fn select_primary_binds(
    binds: &HashMap<(u32, TargetId), TargetBindState>,
) -> HashMap<u32, TargetBindState> {
    let mut per_realm: HashMap<u32, TargetBindState> = HashMap::new();
    for bind in binds.values() {
        per_realm
            .entry(bind.realm_id)
            .and_modify(|existing| {
                if bind.target_id.0 < existing.target_id.0 {
                    *existing = bind.clone();
                }
            })
            .or_insert_with(|| bind.clone());
    }
    per_realm
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
    let size = target
        .size_override
        .or_else(|| match target.kind {
            TargetKind::Window | TargetKind::ViewportEmbed | TargetKind::PanelEmbed => target
                .owner_window_id
                .and_then(|window_id| engine_state.window.states.get(&window_id))
                .map(|state| state.inner_size),
            TargetKind::Texture => None,
        })
        .unwrap_or_else(|| glam::UVec2::new(1, 1));

    SurfaceState {
        kind: match target.kind {
            TargetKind::Window | TargetKind::ViewportEmbed | TargetKind::PanelEmbed => {
                SurfaceKind::Onscreen
            }
            TargetKind::Texture => SurfaceKind::Offscreen,
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
