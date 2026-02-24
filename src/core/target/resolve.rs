use std::collections::{HashMap, HashSet};

use crate::core::realm::{
    AutoLink, ConnectorId, ConnectorState, PresentState, RealmId, RealmKind, SurfaceKind,
    SurfaceState, UniversalState,
};
use crate::core::state::EngineState;
use crate::core::system::push_error_event;
use crate::core::target::{TargetId, TargetKind, TargetLayerLayout, TargetLayerState, TargetState};

const INPUT_FLAG_RAYCAST: u32 = 1 << 0;
pub const INPUT_FLAG_WIDGET_VIEW: u32 = 1 << 1;
const DEFAULT_CH_WIDTH: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
struct ResolvedLayerLayout {
    rect: glam::Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<glam::Vec4>,
}

pub fn sync_auto_graph(engine_state: &mut EngineState) {
    rebuild_target_indexes(&mut engine_state.universal_state);

    let mut desired_layers: Vec<TargetLayerState> = engine_state
        .universal_state
        .target_layers
        .entries
        .values()
        .cloned()
        .collect();
    desired_layers.sort_by_key(|layer| (layer.realm_id, layer.target_id.0));
    let desired_keys: HashSet<(u32, TargetId)> = desired_layers
        .iter()
        .map(|layer| (layer.realm_id, layer.target_id))
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
    let mut auto_link_failures = Vec::new();

    for layer in desired_layers {
        let key = (layer.realm_id, layer.target_id);
        let realm_id = RealmId(layer.realm_id);
        let (realm_kind, mut surface_id) =
            match engine_state.universal_state.realms.entries.get(&realm_id) {
                Some(entry) => (entry.value.kind, entry.value.output_surface),
                None => {
                    auto_link_failures.push(crate::core::realm::TargetAutoLinkFailure {
                        realm_id: layer.realm_id,
                        target_id: layer.target_id.0,
                        reason: "realm-not-found".into(),
                    });
                    remove_auto_link(&mut engine_state.universal_state, key);
                    continue;
                }
            };
        let target = match engine_state
            .universal_state
            .targets
            .entries
            .get(&layer.target_id)
        {
            Some(target) => target.clone(),
            None => {
                auto_link_failures.push(crate::core::realm::TargetAutoLinkFailure {
                    realm_id: layer.realm_id,
                    target_id: layer.target_id.0,
                    reason: "target-not-found".into(),
                });
                remove_auto_link(&mut engine_state.universal_state, key);
                continue;
            }
        };
        let resolved_layout = resolve_layer_layout(engine_state, &target, &layer.layout);

        let primary_target = primary_targets
            .entry(layer.realm_id)
            .or_insert(layer.target_id);
        let is_primary = *primary_target == layer.target_id;

        if surface_id.is_none() {
            let desired_surface = surface_state_for_target(engine_state, &target, Some(&layer));
            surface_id = Some(engine_state.universal_state.surfaces.alloc(desired_surface));
            if let Some(entry) = engine_state
                .universal_state
                .realms
                .entries
                .get_mut(&realm_id)
            {
                entry.value.output_surface = surface_id;
            }
        } else if is_primary {
            let desired_surface = surface_state_for_target(engine_state, &target, Some(&layer));
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
            let host_realm = target
                .window_id
                .and_then(|window_id| {
                    engine_state
                        .universal_state
                        .host_realm_index
                        .get(&window_id)
                })
                .copied();
            let is_host_layer = host_realm == Some(RealmId(layer.realm_id));
            let expects_present = matches!(target.kind, TargetKind::Window) && is_host_layer;
            let expects_connector = matches!(
                target.kind,
                TargetKind::WidgetRealmViewport | TargetKind::RealmPlane
            ) || (matches!(target.kind, TargetKind::Window)
                && !is_host_layer);
            let needs_rebuild = link.surface_id != surface_id
                || (expects_present && link.present_id.is_none())
                || (expects_connector && link.connector_id.is_none());

            if needs_rebuild {
                remove_auto_link(&mut engine_state.universal_state, key);
            } else {
                if let Some(connector_id) = link.connector_id {
                    update_auto_link_layout(
                        &mut engine_state.universal_state,
                        Some(connector_id),
                        target.kind,
                        realm_kind,
                        resolved_layout,
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
                    let host_realm = engine_state
                        .universal_state
                        .host_realm_index
                        .get(&window_id)
                        .copied();
                    if host_realm == Some(realm_id) {
                        present_id =
                            Some(engine_state.universal_state.presents.alloc(PresentState {
                                window_id,
                                surface: surface_id,
                            }));
                    } else if let Some(host_realm) = host_realm {
                        connector_id = Some(engine_state.universal_state.connectors.alloc(
                            ConnectorState {
                                target_realm: host_realm,
                                source_surface: surface_id,
                                rect: resolved_layout.rect,
                                z_index: resolved_layout.z_index,
                                blend_mode: resolved_layout.blend_mode,
                                clip: resolved_layout.clip,
                                input_flags: infer_layer_input_flags(target.kind, realm_kind),
                            },
                        ));
                    } else {
                        auto_link_failures.push(crate::core::realm::TargetAutoLinkFailure {
                            realm_id: layer.realm_id,
                            target_id: layer.target_id.0,
                            reason: "host-realm-not-found".into(),
                        });
                    }
                }
            }
            TargetKind::WidgetRealmViewport => {
                if let Some(window_id) = target.window_id {
                    if let Some(host_realm) = engine_state
                        .universal_state
                        .host_realm_index
                        .get(&window_id)
                        .copied()
                    {
                        let input_flags = infer_layer_input_flags(target.kind, realm_kind);
                        connector_id = Some(engine_state.universal_state.connectors.alloc(
                            ConnectorState {
                                target_realm: host_realm,
                                source_surface: surface_id,
                                rect: resolved_layout.rect,
                                z_index: resolved_layout.z_index,
                                blend_mode: resolved_layout.blend_mode,
                                clip: resolved_layout.clip,
                                input_flags,
                            },
                        ));
                    } else {
                        auto_link_failures.push(crate::core::realm::TargetAutoLinkFailure {
                            realm_id: layer.realm_id,
                            target_id: layer.target_id.0,
                            reason: "host-realm-not-found".into(),
                        });
                    }
                }
            }
            TargetKind::RealmPlane => {
                if let Some(window_id) = target.window_id {
                    if let Some(host_realm) = engine_state
                        .universal_state
                        .host_realm_index
                        .get(&window_id)
                        .copied()
                    {
                        connector_id = Some(engine_state.universal_state.connectors.alloc(
                            ConnectorState {
                                target_realm: host_realm,
                                source_surface: surface_id,
                                rect: resolved_layout.rect,
                                z_index: resolved_layout.z_index,
                                blend_mode: resolved_layout.blend_mode,
                                clip: resolved_layout.clip,
                                input_flags: infer_layer_input_flags(target.kind, realm_kind),
                            },
                        ));
                    } else {
                        auto_link_failures.push(crate::core::realm::TargetAutoLinkFailure {
                            realm_id: layer.realm_id,
                            target_id: layer.target_id.0,
                            reason: "host-realm-not-found".into(),
                        });
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
    if auto_link_failures != engine_state.universal_state.target_autolink_failures {
        for failure in &auto_link_failures {
            push_error_event(
                engine_state,
                "target-auto-link",
                format!(
                    "realm_id={} target_id={} reason={}",
                    failure.realm_id, failure.target_id, failure.reason
                ),
                None,
                None,
            );
        }
    }
    engine_state.universal_state.target_autolink_failures = auto_link_failures;
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
    target_kind: TargetKind,
    source_realm_kind: RealmKind,
    layout: ResolvedLayerLayout,
) {
    let Some(connector_id) = connector_id else {
        return;
    };
    let Some(entry) = universal.connectors.get_mut(connector_id) else {
        return;
    };

    entry.value.rect = layout.rect;
    entry.value.z_index = layout.z_index;
    entry.value.blend_mode = layout.blend_mode;
    entry.value.clip = layout.clip;
    entry.value.input_flags = infer_layer_input_flags(target_kind, source_realm_kind);
}

fn infer_layer_input_flags(target_kind: TargetKind, source_realm_kind: RealmKind) -> u32 {
    match target_kind {
        TargetKind::WidgetRealmViewport => {
            let mut flags = INPUT_FLAG_WIDGET_VIEW;
            if source_realm_kind == RealmKind::ThreeD {
                flags |= INPUT_FLAG_RAYCAST;
            }
            flags
        }
        TargetKind::Window if source_realm_kind == RealmKind::ThreeD => INPUT_FLAG_RAYCAST,
        TargetKind::RealmPlane | TargetKind::Window | TargetKind::Texture => 0,
    }
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

fn rebuild_target_indexes(universal: &mut UniversalState) {
    universal.host_realm_index.clear();
    for (realm_id, entry) in &universal.realms.entries {
        let Some(window_id) = entry.value.host_window_id else {
            continue;
        };
        match universal.host_realm_index.get_mut(&window_id) {
            Some(existing) => {
                if realm_id.0 < existing.0 {
                    *existing = *realm_id;
                }
            }
            None => {
                universal.host_realm_index.insert(window_id, *realm_id);
            }
        }
    }

    universal.target_ui_realm_index.clear();
    for layer in universal.target_layers.entries.values() {
        let realm_id = RealmId(layer.realm_id);
        let Some(realm_entry) = universal.realms.entries.get(&realm_id) else {
            continue;
        };
        if realm_entry.value.kind != crate::core::realm::RealmKind::TwoD {
            continue;
        }
        match universal.target_ui_realm_index.get_mut(&layer.target_id) {
            Some(existing) => {
                if realm_id.0 < existing.0 {
                    *existing = realm_id;
                }
            }
            None => {
                universal
                    .target_ui_realm_index
                    .insert(layer.target_id, realm_id);
            }
        }
    }
}

fn surface_state_for_target(
    engine_state: &EngineState,
    target: &TargetState,
    layer: Option<&TargetLayerState>,
) -> SurfaceState {
    let layer_size = layer.and_then(|layer| {
        let resolved = resolve_layer_layout(engine_state, target, &layer.layout);
        let width = resolved.rect.z.max(1.0).round() as u32;
        let height = resolved.rect.w.max(1.0).round() as u32;
        if width > 0 && height > 0 {
            Some(glam::UVec2::new(width, height))
        } else {
            None
        }
    });
    let size = match target.kind {
        TargetKind::Texture => target.size.unwrap_or_else(|| glam::UVec2::new(1, 1)),
        TargetKind::Window => {
            let is_window_connector = layer
                .and_then(|layer| {
                    target
                        .window_id
                        .map(|window_id| (layer.realm_id, window_id))
                })
                .and_then(|(layer_realm_id, window_id)| {
                    engine_state
                        .universal_state
                        .host_realm_index
                        .get(&window_id)
                        .map(|host_realm| host_realm.0 != layer_realm_id)
                })
                .unwrap_or(false);
            if is_window_connector {
                layer_size
                    .or_else(|| {
                        target
                            .window_id
                            .and_then(|window_id| engine_state.window.states.get(&window_id))
                            .map(|state| state.inner_size)
                    })
                    .unwrap_or_else(|| glam::UVec2::new(1, 1))
            } else {
                target
                    .window_id
                    .and_then(|window_id| engine_state.window.states.get(&window_id))
                    .map(|state| state.inner_size)
                    .unwrap_or_else(|| glam::UVec2::new(1, 1))
            }
        }
        TargetKind::WidgetRealmViewport | TargetKind::RealmPlane => target
            .size
            .or(layer_size)
            .or_else(|| {
                target
                    .window_id
                    .and_then(|window_id| engine_state.window.states.get(&window_id))
                    .map(|state| state.inner_size)
            })
            .unwrap_or_else(|| glam::UVec2::new(1, 1)),
    };

    SurfaceState {
        kind: match target.kind {
            TargetKind::Window => SurfaceKind::Onscreen,
            TargetKind::WidgetRealmViewport | TargetKind::RealmPlane | TargetKind::Texture => {
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

fn resolve_layer_layout(
    engine_state: &EngineState,
    target: &TargetState,
    layout: &TargetLayerLayout,
) -> ResolvedLayerLayout {
    let ref_size = target
        .window_id
        .and_then(|window_id| engine_state.window.states.get(&window_id))
        .map(|state| state.inner_size)
        .unwrap_or_else(|| glam::UVec2::new(1, 1));
    let ref_width = ref_size.x.max(1) as f32;
    let ref_height = ref_size.y.max(1) as f32;
    let left = layout.left.resolve(ref_width, DEFAULT_CH_WIDTH);
    let top = layout.top.resolve(ref_height, DEFAULT_CH_WIDTH);
    let width = layout.width.resolve(ref_width, DEFAULT_CH_WIDTH).max(0.0);
    let height = layout.height.resolve(ref_height, DEFAULT_CH_WIDTH).max(0.0);
    ResolvedLayerLayout {
        rect: glam::Vec4::new(left, top, width, height),
        z_index: layout.z_index,
        blend_mode: layout.blend_mode,
        clip: layout.clip,
    }
}
