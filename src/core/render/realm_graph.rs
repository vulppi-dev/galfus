use std::collections::{HashMap, HashSet};

use crate::core::realm::{ConnectorId, RealmGraphPlan, RealmId, SurfaceId, UniversalState};
use crate::core::render::{RenderState, passes};
use crate::core::resources::RenderTarget;
use crate::core::window::WindowState;

#[derive(Debug, Clone)]
pub(crate) struct SurfaceSnapshot {
    pub(crate) view: wgpu::TextureView,
    pub(crate) size: glam::UVec2,
}

pub(crate) fn collect_cut_connectors(plan: &RealmGraphPlan) -> HashSet<ConnectorId> {
    vulfram_render::collect_cut_connectors(plan)
}

pub(crate) fn update_surface_cache(
    universal: &mut UniversalState,
    _cut_connectors: &HashSet<ConnectorId>,
) {
    let connectors: Vec<_> = universal
        .connectors
        .entries
        .iter()
        .map(|(connector_id, connector)| (*connector_id, connector.value.source_surface))
        .collect();
    vulfram_render::update_surface_cache(&mut universal.surface_cache, &connectors);
}

pub(crate) fn map_realms_to_windows(universal: &UniversalState) -> HashMap<RealmId, u32> {
    let existing_realms = universal.realms.entries.keys().copied().collect();
    let layer_windows: Vec<_> = universal
        .target_layers
        .entries
        .values()
        .filter_map(|layer| {
            let target = universal.targets.entries.get(&layer.target_id)?;
            Some((RealmId(layer.realm_id), target.window_id?))
        })
        .collect();
    let presents: Vec<_> = universal
        .presents
        .entries
        .values()
        .map(|entry| (entry.value.surface, entry.value.window_id))
        .collect();
    let realm_output_surfaces: HashMap<_, _> = universal
        .realms
        .entries
        .iter()
        .map(|(realm_id, entry)| (*realm_id, entry.value.output_surface))
        .collect();
    vulfram_render::map_realms_to_windows(
        &existing_realms,
        &layer_windows,
        &presents,
        &realm_output_surfaces,
    )
}

pub(crate) fn collect_present_sizes(
    universal: &UniversalState,
    windows: &HashMap<u32, WindowState>,
    cache: &mut HashMap<SurfaceId, glam::UVec2>,
    cache_hash: &mut u64,
) {
    let presents: Vec<_> = universal
        .presents
        .entries
        .values()
        .map(|entry| (entry.value.surface, entry.value.window_id))
        .collect();
    let window_sizes = windows
        .iter()
        .map(|(window_id, window)| (*window_id, window.inner_size))
        .collect();
    let _ = vulfram_render::update_present_size_cache(&presents, &window_sizes, cache, cache_hash);
}

pub(crate) fn collect_connectors_by_realm(
    universal: &UniversalState,
) -> HashMap<RealmId, Vec<ConnectorId>> {
    let connectors: Vec<_> = universal
        .connectors
        .entries
        .iter()
        .map(|(connector_id, entry)| (*connector_id, entry.value.target_realm))
        .collect();
    vulfram_render::collect_connectors_by_realm(&connectors)
}

pub(crate) fn collect_surface_views(
    device: &wgpu::Device,
    universal: &UniversalState,
    surface_targets: &mut HashMap<SurfaceId, RenderTarget>,
    present_sizes: &HashMap<SurfaceId, glam::UVec2>,
) -> HashMap<SurfaceId, SurfaceSnapshot> {
    surface_targets.retain(|surface_id, _| universal.surfaces.entries.contains_key(surface_id));

    let mut views = HashMap::new();

    for (surface_id, entry) in universal.surfaces.entries.iter() {
        let mut target_size = entry.value.size;
        if entry.value.kind == crate::core::realm::SurfaceKind::Onscreen {
            if let Some(present_size) = present_sizes.get(surface_id) {
                target_size = *present_size;
            }
        }
        let target_format = entry
            .value
            .format_policy
            .unwrap_or(wgpu::TextureFormat::Rgba16Float);

        let surface_target = ensure_surface_target(
            device,
            surface_targets,
            *surface_id,
            target_size,
            target_format,
        );

        views.insert(
            *surface_id,
            SurfaceSnapshot {
                view: surface_target.view.clone(),
                size: target_size,
            },
        );
    }

    views
}

pub(crate) fn resolve_realm_surface(
    universal: &UniversalState,
    realm_id: RealmId,
) -> Option<SurfaceId> {
    let realm_output_surfaces: HashMap<_, _> = universal
        .realms
        .entries
        .iter()
        .map(|(realm_id, entry)| (*realm_id, entry.value.output_surface))
        .collect();
    vulfram_render::resolve_realm_surface(&realm_output_surfaces, realm_id)
}

pub(crate) fn ensure_surface_target<'a>(
    device: &wgpu::Device,
    surface_targets: &'a mut HashMap<SurfaceId, RenderTarget>,
    surface_id: SurfaceId,
    size: glam::UVec2,
    format: wgpu::TextureFormat,
) -> &'a RenderTarget {
    let size = glam::UVec2::new(size.x.max(1), size.y.max(1));
    let needs_target = match surface_targets.get(&surface_id) {
        Some(existing) => {
            let tex_size = existing.texture.size();
            tex_size.width != size.x || tex_size.height != size.y || existing.format != format
        }
        None => true,
    };

    if needs_target {
        let extent = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        surface_targets.insert(surface_id, RenderTarget::new(device, extent, format));
    }

    surface_targets
        .get(&surface_id)
        .expect("surface target missing after ensure")
}

pub(crate) fn compose_realm_connectors(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    universal: &UniversalState,
    connectors_by_realm: &HashMap<RealmId, Vec<ConnectorId>>,
    realm_id: RealmId,
    target_surface: SurfaceId,
    cut_connectors: &HashSet<ConnectorId>,
    surface_views: &HashMap<SurfaceId, SurfaceSnapshot>,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    frame_report: &mut crate::core::realm::FrameReport,
) {
    let mut overlays = Vec::new();

    let Some(connector_ids) = connectors_by_realm.get(&realm_id) else {
        return;
    };

    for connector_id in connector_ids {
        let Some(connector) = universal.connectors.entries.get(connector_id) else {
            continue;
        };
        if (connector.value.input_flags & crate::core::target::resolve::INPUT_FLAG_WIDGET_VIEW) != 0
        {
            continue;
        }
        if connector.value.source_surface == target_surface {
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.self_sampled_connectors,
                connector_id.0,
            );
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.no_progress_realms,
                realm_id.0,
            );
            continue;
        }
        let source_surface = resolve_connector_surface(
            universal,
            *connector_id,
            cut_connectors,
            connector.value.source_surface,
        );
        let Some(source_surface) = source_surface else {
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.blocked_connectors,
                connector_id.0,
            );
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.no_progress_realms,
                realm_id.0,
            );
            continue;
        };
        let Some(snapshot) = surface_views.get(&source_surface) else {
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.blocked_connectors,
                connector_id.0,
            );
            crate::core::realm::FrameReport::push_unique(
                &mut frame_report.no_progress_realms,
                realm_id.0,
            );
            continue;
        };
        overlays.push((
            connector.value.z_index,
            passes::ComposeOverlay {
                source_view: &snapshot.view,
                source_size: snapshot.size,
                rect: connector.value.rect,
                clip: connector.value.clip,
                blend: blend_state_for_mode(connector.value.blend_mode),
            },
        ));
    }

    overlays.sort_by_key(|(z_index, _)| *z_index);
    let ordered: Vec<_> = overlays.into_iter().map(|(_, overlay)| overlay).collect();

    passes::pass_compose_overlays(
        render_state,
        device,
        encoder,
        target_view,
        target_format,
        target_size,
        &ordered,
        frame_index,
    );
}

fn resolve_connector_surface(
    universal: &UniversalState,
    connector_id: ConnectorId,
    cut_connectors: &HashSet<ConnectorId>,
    default_surface: SurfaceId,
) -> Option<SurfaceId> {
    if !cut_connectors.contains(&connector_id) {
        return Some(default_surface);
    }
    Some(
        universal
            .surface_cache
            .last_good
            .get(&connector_id)
            .copied()
            .or_else(|| universal.surface_cache.fallback.get(&connector_id).copied())
            .unwrap_or(default_surface),
    )
}

fn blend_state_for_mode(mode: u32) -> Option<wgpu::BlendState> {
    match mode {
        0 => Some(wgpu::BlendState::ALPHA_BLENDING),
        1 => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        2 => None,
        _ => Some(wgpu::BlendState::ALPHA_BLENDING),
    }
}
