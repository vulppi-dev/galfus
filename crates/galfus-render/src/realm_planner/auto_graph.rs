use std::collections::HashMap;

use galfus_realm_core::{RealmId, RealmKind, SurfaceId, TargetId, TargetKind, TargetLayerLayout};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoGraphResolvedLayout {
    pub rect: glam::Vec4,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<glam::Vec4>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGraphSurfaceKind {
    Onscreen,
    Offscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoGraphSurfaceSpec {
    pub kind: AutoGraphSurfaceKind,
    pub size: glam::UVec2,
    pub format_policy: Option<wgpu::TextureFormat>,
    pub alpha_policy: Option<wgpu::CompositeAlphaMode>,
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGraphLinkPlan {
    None,
    Present {
        window_id: u32,
    },
    Connector {
        target_realm: RealmId,
        input_flags: u32,
    },
}

pub const AUTO_GRAPH_INPUT_FLAG_RAYCAST: u32 = 1 << 0;
pub const AUTO_GRAPH_INPUT_FLAG_WIDGET_VIEW: u32 = 1 << 1;
const DEFAULT_CH_WIDTH: f32 = 8.0;

pub fn resolve_auto_graph_layout(
    reference_size: glam::UVec2,
    layout: &TargetLayerLayout,
) -> AutoGraphResolvedLayout {
    let ref_width = reference_size.x.max(1) as f32;
    let ref_height = reference_size.y.max(1) as f32;
    let left = layout.left.resolve(ref_width, DEFAULT_CH_WIDTH);
    let top = layout.top.resolve(ref_height, DEFAULT_CH_WIDTH);
    let width = layout.width.resolve(ref_width, DEFAULT_CH_WIDTH).max(0.0);
    let height = layout.height.resolve(ref_height, DEFAULT_CH_WIDTH).max(0.0);

    AutoGraphResolvedLayout {
        rect: glam::Vec4::new(left, top, width, height),
        z_index: layout.z_index,
        blend_mode: layout.blend_mode,
        clip: layout.clip,
    }
}

pub fn infer_auto_graph_input_flags(target_kind: TargetKind, source_realm_kind: RealmKind) -> u32 {
    match target_kind {
        TargetKind::Window if source_realm_kind == RealmKind::ThreeD => {
            AUTO_GRAPH_INPUT_FLAG_RAYCAST
        }
        TargetKind::Window | TargetKind::Texture => 0,
    }
}

pub fn plan_host_realm_index(
    presents: &[(u32, SurfaceId)],
    realm_output_surfaces: &HashMap<RealmId, Option<SurfaceId>>,
    layers: &HashMap<(u32, TargetId), (TargetKind, Option<u32>)>,
) -> HashMap<u32, RealmId> {
    let mut host_realm_index: HashMap<u32, RealmId> = HashMap::new();

    for (window_id, present_surface) in presents {
        let Some(present_realm) =
            realm_output_surfaces
                .iter()
                .find_map(|(realm_id, surface_id)| {
                    (*surface_id == Some(*present_surface)).then_some(*realm_id)
                })
        else {
            continue;
        };

        match host_realm_index.get_mut(window_id) {
            Some(existing) => {
                if present_realm.0 < existing.0 {
                    *existing = present_realm;
                }
            }
            None => {
                host_realm_index.insert(*window_id, present_realm);
            }
        }
    }

    for ((realm_id, _target_id), (target_kind, window_id)) in layers {
        if *target_kind != TargetKind::Window {
            continue;
        }
        let Some(window_id) = *window_id else {
            continue;
        };
        let realm_id = RealmId(*realm_id);
        if !realm_output_surfaces.contains_key(&realm_id) {
            continue;
        }
        if host_realm_index.contains_key(&window_id) {
            continue;
        }
        match host_realm_index.get_mut(&window_id) {
            Some(existing) => {
                if realm_id.0 < existing.0 {
                    *existing = realm_id;
                }
            }
            None => {
                host_realm_index.insert(window_id, realm_id);
            }
        }
    }

    host_realm_index
}

pub fn plan_auto_graph_surface_spec(
    target_kind: TargetKind,
    target_window_id: Option<u32>,
    target_size: Option<glam::UVec2>,
    format_policy: Option<wgpu::TextureFormat>,
    alpha_policy: Option<wgpu::CompositeAlphaMode>,
    msaa_samples: Option<u32>,
    layer_layout: Option<&TargetLayerLayout>,
    layer_realm_id: Option<u32>,
    host_realm_index: &HashMap<u32, RealmId>,
    window_sizes: &HashMap<u32, glam::UVec2>,
) -> AutoGraphSurfaceSpec {
    let reference_size = target_window_id
        .and_then(|window_id| window_sizes.get(&window_id).copied())
        .unwrap_or_else(|| glam::UVec2::new(1, 1));
    let resolved_layout =
        layer_layout.map(|layout| resolve_auto_graph_layout(reference_size, layout));
    let layer_size = resolved_layout.map(|resolved| {
        glam::UVec2::new(
            resolved.rect.z.max(1.0).round() as u32,
            resolved.rect.w.max(1.0).round() as u32,
        )
    });

    let is_window_connector = target_window_id
        .and_then(|window_id| host_realm_index.get(&window_id).copied())
        .zip(layer_realm_id.map(RealmId))
        .map(|(host_realm, layer_realm)| host_realm != layer_realm)
        .unwrap_or(false);

    let size = match target_kind {
        TargetKind::Texture => target_size.unwrap_or_else(|| glam::UVec2::new(1, 1)),
        TargetKind::Window => {
            if is_window_connector {
                layer_size
                    .or_else(|| {
                        target_window_id.and_then(|window_id| window_sizes.get(&window_id).copied())
                    })
                    .unwrap_or_else(|| glam::UVec2::new(1, 1))
            } else {
                target_window_id
                    .and_then(|window_id| window_sizes.get(&window_id).copied())
                    .unwrap_or_else(|| glam::UVec2::new(1, 1))
            }
        }
    };

    let kind = match target_kind {
        TargetKind::Window if !is_window_connector => AutoGraphSurfaceKind::Onscreen,
        TargetKind::Window | TargetKind::Texture => AutoGraphSurfaceKind::Offscreen,
    };

    AutoGraphSurfaceSpec {
        kind,
        size,
        format_policy,
        alpha_policy,
        msaa_samples,
    }
}

pub fn plan_auto_graph_link(
    target_kind: TargetKind,
    target_window_id: Option<u32>,
    source_realm_id: RealmId,
    source_realm_kind: RealmKind,
    host_realm_index: &HashMap<u32, RealmId>,
) -> AutoGraphLinkPlan {
    match target_kind {
        TargetKind::Window => {
            let Some(window_id) = target_window_id else {
                return AutoGraphLinkPlan::None;
            };
            let Some(host_realm) = host_realm_index.get(&window_id).copied() else {
                return AutoGraphLinkPlan::None;
            };
            if host_realm == source_realm_id {
                AutoGraphLinkPlan::Present { window_id }
            } else {
                AutoGraphLinkPlan::Connector {
                    target_realm: host_realm,
                    input_flags: infer_auto_graph_input_flags(target_kind, source_realm_kind),
                }
            }
        }
        TargetKind::Texture => AutoGraphLinkPlan::None,
    }
}
