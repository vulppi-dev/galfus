use std::collections::{HashMap, HashSet};

use crate::core::realm::{
    AutoLink, ConnectorId, ConnectorState, PresentState, RealmId, RealmKind, SurfaceKind,
    SurfaceState, UniversalState,
};
use crate::core::state::EngineState;
use crate::core::system::push_error_event;
use crate::core::target::{TargetId, TargetKind, TargetLayerLayout, TargetLayerState, TargetState};

pub const INPUT_FLAG_WIDGET_VIEW: u32 = vulfram_render::AUTO_GRAPH_INPUT_FLAG_WIDGET_VIEW;

#[derive(Debug, Clone, Copy)]
struct ResolvedLayerLayout {
    rect: glam::Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<glam::Vec4>,
}

#[derive(Debug, Clone)]
struct PlannedLayerSync {
    key: (u32, TargetId),
    layer: TargetLayerState,
    target: TargetState,
    realm_kind: RealmKind,
    current_surface_id: Option<crate::core::realm::SurfaceId>,
    resolved_layout: ResolvedLayerLayout,
    is_primary: bool,
}

#[derive(Debug, Default)]
struct AutoGraphSyncPlan {
    removed_keys: Vec<(u32, TargetId)>,
    layer_syncs: Vec<PlannedLayerSync>,
    auto_link_failures: Vec<crate::core::realm::TargetAutoLinkFailure>,
}

pub fn sync_auto_graph(engine_state: &mut EngineState) {
    refresh_target_indexes(&mut engine_state.universal_state);

    let plan = plan_auto_graph_sync(engine_state);
    apply_auto_graph_sync(engine_state, plan);
}

fn plan_auto_graph_sync(engine_state: &EngineState) -> AutoGraphSyncPlan {
    let mut plan = AutoGraphSyncPlan::default();

    let mut desired_layers: Vec<TargetLayerState> = engine_state
        .universal_state
        .targets
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
        .targets
        .auto_links
        .keys()
        .copied()
        .collect();
    for key in existing_keys {
        if !desired_keys.contains(&key) {
            plan.removed_keys.push(key);
        }
    }

    let mut primary_targets: HashMap<u32, TargetId> = HashMap::new();

    for layer in desired_layers {
        let key = (layer.realm_id, layer.target_id);
        let realm_id = RealmId(layer.realm_id);
        let (realm_kind, surface_id) = match engine_state
            .universal_state
            .composition
            .realms
            .entries
            .get(&realm_id)
        {
            Some(entry) => (entry.value.kind, entry.value.output_surface),
            None => {
                plan.auto_link_failures
                    .push(crate::core::realm::TargetAutoLinkFailure {
                        realm_id: layer.realm_id,
                        target_id: layer.target_id.0,
                        reason: "realm-not-found".into(),
                    });
                plan.removed_keys.push(key);
                continue;
            }
        };
        let target = match engine_state
            .universal_state
            .targets
            .targets
            .entries
            .get(&layer.target_id)
        {
            Some(target) => target.clone(),
            None => {
                plan.auto_link_failures
                    .push(crate::core::realm::TargetAutoLinkFailure {
                        realm_id: layer.realm_id,
                        target_id: layer.target_id.0,
                        reason: "target-not-found".into(),
                    });
                plan.removed_keys.push(key);
                continue;
            }
        };
        let resolved_layout = resolve_layer_layout(engine_state, &target, &layer.layout);

        let primary_target = primary_targets
            .entry(layer.realm_id)
            .or_insert(layer.target_id);
        let is_primary = *primary_target == layer.target_id;

        plan.layer_syncs.push(PlannedLayerSync {
            key,
            layer,
            target,
            realm_kind,
            current_surface_id: surface_id,
            resolved_layout,
            is_primary,
        });
    }

    plan
}

fn apply_auto_graph_sync(engine_state: &mut EngineState, plan: AutoGraphSyncPlan) {
    for key in plan.removed_keys {
        remove_auto_link(&mut engine_state.universal_state, key);
    }

    for planned in plan.layer_syncs {
        apply_planned_layer_sync(engine_state, planned);
    }

    if plan.auto_link_failures
        != engine_state
            .universal_state
            .targets
            .target_autolink_failures
    {
        for failure in &plan.auto_link_failures {
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
    engine_state
        .universal_state
        .targets
        .target_autolink_failures = plan.auto_link_failures;
}

fn apply_planned_layer_sync(engine_state: &mut EngineState, planned: PlannedLayerSync) {
    let key = planned.key;
    let realm_id = RealmId(planned.layer.realm_id);
    let desired_surface =
        surface_state_for_target(engine_state, &planned.target, Some(&planned.layer));
    let mut surface_id = planned.current_surface_id;

    if surface_id.is_none() {
        surface_id = Some(
            engine_state
                .universal_state
                .composition
                .surfaces
                .alloc(desired_surface),
        );
        if let Some(entry) = engine_state
            .universal_state
            .composition
            .realms
            .entries
            .get_mut(&realm_id)
        {
            entry.value.output_surface = surface_id;
        }
    } else if planned.is_primary {
        if let Some(surface_id) = surface_id {
            if let Some(entry) = engine_state
                .universal_state
                .composition
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
        return;
    };

    if let Some(link) = engine_state
        .universal_state
        .targets
        .auto_links
        .get(&key)
        .cloned()
    {
        let host_realm = planned
            .target
            .window_id
            .and_then(|window_id| {
                engine_state
                    .universal_state
                    .targets
                    .host_realm_index
                    .get(&window_id)
            })
            .copied();
        let is_host_layer = host_realm == Some(RealmId(planned.layer.realm_id));
        let expects_present = matches!(planned.target.kind, TargetKind::Window) && is_host_layer;
        let expects_connector = matches!(
            planned.target.kind,
            TargetKind::WidgetRealmViewport | TargetKind::RealmPlane
        ) || (matches!(planned.target.kind, TargetKind::Window)
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
                    planned.target.kind,
                    planned.realm_kind,
                    planned.resolved_layout,
                );
            }
            return;
        }
    }

    let link_plan = vulfram_render::plan_auto_graph_link(
        planned.target.kind,
        planned.target.window_id,
        realm_id,
        planned.realm_kind,
        &engine_state.universal_state.targets.host_realm_index,
    );
    let mut connector_id = None;
    let mut present_id = None;
    match link_plan {
        vulfram_render::AutoGraphLinkPlan::None => {}
        vulfram_render::AutoGraphLinkPlan::Present { window_id } => {
            present_id = Some(engine_state.universal_state.composition.presents.alloc(
                PresentState {
                    window_id,
                    surface: surface_id,
                },
            ));
        }
        vulfram_render::AutoGraphLinkPlan::Connector {
            target_realm,
            input_flags,
        } => {
            connector_id = Some(engine_state.universal_state.composition.connectors.alloc(
                ConnectorState {
                    target_realm,
                    source_surface: surface_id,
                    rect: planned.resolved_layout.rect,
                    z_index: planned.resolved_layout.z_index,
                    blend_mode: planned.resolved_layout.blend_mode,
                    clip: planned.resolved_layout.clip,
                    input_flags,
                },
            ));
        }
    }

    engine_state.universal_state.targets.auto_links.insert(
        key,
        AutoLink {
            surface_id,
            connector_id,
            present_id,
        },
    );
}

pub fn refresh_target_indexes(universal: &mut UniversalState) {
    rebuild_target_indexes(universal);
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
    let Some(entry) = universal.composition.connectors.get_mut(connector_id) else {
        return;
    };

    entry.value.rect = layout.rect;
    entry.value.z_index = layout.z_index;
    entry.value.blend_mode = layout.blend_mode;
    entry.value.clip = layout.clip;
    entry.value.input_flags = infer_layer_input_flags(target_kind, source_realm_kind);
}

fn infer_layer_input_flags(target_kind: TargetKind, source_realm_kind: RealmKind) -> u32 {
    vulfram_render::infer_auto_graph_input_flags(target_kind, source_realm_kind)
}

fn remove_auto_link(universal: &mut UniversalState, key: (u32, TargetId)) {
    let realm_id = key.0;
    let Some(link) = universal.targets.auto_links.remove(&key) else {
        return;
    };

    if let Some(connector_id) = link.connector_id {
        universal.composition.connectors.remove(connector_id);
        universal
            .interaction
            .input_routing
            .captures
            .retain(|_, capture| capture.connector_id != connector_id.0);
        universal
            .composition
            .surface_cache
            .last_good
            .remove(&connector_id);
        universal
            .composition
            .surface_cache
            .fallback
            .remove(&connector_id);
    }
    if let Some(present_id) = link.present_id {
        universal.composition.presents.remove(present_id);
    }
    if let Some(entry) = universal
        .composition
        .realms
        .entries
        .get_mut(&RealmId(realm_id))
    {
        if entry.value.output_surface == Some(link.surface_id) {
            let surface_id = link.surface_id;
            let still_used = universal
                .targets
                .auto_links
                .iter()
                .any(|((realm, _), link)| *realm == realm_id && link.surface_id == surface_id);
            if !still_used {
                entry.value.output_surface = None;
            }
        }
    }

    let surface_still_used = universal
        .targets
        .auto_links
        .values()
        .any(|link_entry| link_entry.surface_id == link.surface_id);
    if !surface_still_used {
        universal.composition.surfaces.remove(link.surface_id);
        universal
            .composition
            .surface_cache
            .last_good
            .retain(|_, source| *source != link.surface_id);
        universal
            .composition
            .surface_cache
            .fallback
            .retain(|_, source| *source != link.surface_id);
    }
}

fn rebuild_target_indexes(universal: &mut UniversalState) {
    let presents: Vec<_> = universal
        .composition
        .presents
        .entries
        .values()
        .map(|entry| (entry.value.window_id, entry.value.surface))
        .collect();
    let realm_output_surfaces: HashMap<_, _> = universal
        .composition
        .realms
        .entries
        .iter()
        .map(|(realm_id, entry)| (*realm_id, entry.value.output_surface))
        .collect();
    let layer_target_kinds: HashMap<_, _> = universal
        .targets
        .target_layers
        .entries
        .iter()
        .filter_map(|((realm_id, target_id), _layer)| {
            let target = universal.targets.targets.entries.get(target_id)?;
            Some(((*realm_id, *target_id), (target.kind, target.window_id)))
        })
        .collect();
    universal.targets.host_realm_index = vulfram_render::plan_host_realm_index(
        &presents,
        &realm_output_surfaces,
        &layer_target_kinds,
    );

    let target_layers: Vec<_> = universal
        .targets
        .target_layers
        .entries
        .keys()
        .map(|(realm_id, target_id)| (*realm_id, *target_id))
        .collect();
    let realm_kinds: HashMap<_, _> = universal
        .composition
        .realms
        .entries
        .iter()
        .map(|(realm_id, entry)| (*realm_id, entry.value.kind))
        .collect();
    universal.targets.target_ui_realm_index =
        vulfram_render::plan_target_ui_realm_index(&target_layers, &realm_kinds);
}

fn surface_state_for_target(
    engine_state: &EngineState,
    target: &TargetState,
    layer: Option<&TargetLayerState>,
) -> SurfaceState {
    let window_sizes: HashMap<_, _> = engine_state
        .window
        .states
        .iter()
        .map(|(window_id, state)| (*window_id, state.inner_size))
        .collect();
    let surface_spec = vulfram_render::plan_auto_graph_surface_spec(
        target.kind,
        target.window_id,
        target.size,
        target.format_policy,
        target.alpha_policy,
        target.msaa_samples,
        layer.map(|layer| &layer.layout),
        layer.map(|layer| layer.realm_id),
        &engine_state.universal_state.targets.host_realm_index,
        &window_sizes,
    );

    SurfaceState {
        kind: match surface_spec.kind {
            vulfram_render::AutoGraphSurfaceKind::Onscreen => SurfaceKind::Onscreen,
            vulfram_render::AutoGraphSurfaceKind::Offscreen => SurfaceKind::Offscreen,
        },
        size: surface_spec.size,
        format_policy: surface_spec.format_policy,
        alpha_policy: surface_spec.alpha_policy,
        msaa_samples: surface_spec.msaa_samples,
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
    let reference_size = target
        .window_id
        .and_then(|window_id| engine_state.window.states.get(&window_id))
        .map(|state| state.inner_size)
        .unwrap_or_else(|| glam::UVec2::new(1, 1));
    let resolved = vulfram_render::resolve_auto_graph_layout(reference_size, layout);
    ResolvedLayerLayout {
        rect: resolved.rect,
        z_index: resolved.z_index,
        blend_mode: resolved.blend_mode,
        clip: resolved.clip,
    }
}
