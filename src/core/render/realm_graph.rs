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
    plan.cut_edges
        .iter()
        .filter_map(|edge| edge.connector_id)
        .collect()
}

pub(crate) fn update_surface_cache(
    universal: &mut UniversalState,
    cut_connectors: &HashSet<ConnectorId>,
) {
    for (connector_id, connector) in universal.connectors.entries.iter() {
        if cut_connectors.contains(connector_id) {
            continue;
        }
        let target_surface = universal
            .realms
            .entries
            .get(&connector.value.target_realm)
            .and_then(|entry| entry.value.output_surface);
        let Some(target_surface) = target_surface else {
            continue;
        };
        universal
            .surface_cache
            .last_good
            .insert(target_surface, connector.value.source_surface);
        universal
            .surface_cache
            .fallback
            .entry(target_surface)
            .or_insert(connector.value.source_surface);
    }
}

pub(crate) fn map_realms_to_windows(universal: &UniversalState) -> HashMap<RealmId, u32> {
    let mut map = HashMap::new();
    for present in universal.presents.entries.values() {
        if let Some(realm_id) = find_realm_by_surface(universal, present.value.surface) {
            map.insert(realm_id, present.value.window_id);
        }
    }
    map
}

pub(crate) fn collect_surface_views(
    device: &wgpu::Device,
    windows: &mut HashMap<u32, WindowState>,
    universal: &UniversalState,
) -> HashMap<SurfaceId, SurfaceSnapshot> {
    let mut views = HashMap::new();
    for present in universal.presents.entries.values() {
        let Some(window_state) = windows.get_mut(&present.value.window_id) else {
            continue;
        };
        let (target_size, target_format) = universal
            .surfaces
            .entries
            .get(&present.value.surface)
            .map(|entry| {
                (
                    entry.value.size,
                    entry.value
                        .format_policy
                        .unwrap_or(wgpu::TextureFormat::Rgba16Float),
                )
            })
            .unwrap_or((window_state.inner_size, wgpu::TextureFormat::Rgba16Float));
        ensure_surface_target(
            device,
            &mut window_state.surface_target,
            target_size,
            target_format,
        );
        let Some(surface_target) = window_state.surface_target.as_ref() else {
            continue;
        };
        let view = surface_target
            ._texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        views.insert(
            present.value.surface,
            SurfaceSnapshot {
                view,
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
    universal
        .realms
        .entries
        .get(&realm_id)
        .and_then(|entry| entry.value.output_surface)
}

pub(crate) fn ensure_surface_target(
    device: &wgpu::Device,
    surface_target: &mut Option<RenderTarget>,
    size: glam::UVec2,
    format: wgpu::TextureFormat,
) {
    crate::core::resources::ensure_render_target(
        device,
        surface_target,
        size.x.max(1),
        size.y.max(1),
        format,
    );
}

pub(crate) fn compose_realm_connectors(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    universal: &UniversalState,
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

    for (connector_id, connector) in universal.connectors.entries.iter() {
        if connector.value.target_realm != realm_id {
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
            target_surface,
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
        let source_size = universal
            .surfaces
            .entries
            .get(&source_surface)
            .map(|entry| entry.value.size)
            .unwrap_or(snapshot.size);
        overlays.push((
            connector.value.z_index,
            passes::ComposeOverlay {
                source_view: &snapshot.view,
                source_size,
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
    target_surface: SurfaceId,
    connector_id: ConnectorId,
    cut_connectors: &HashSet<ConnectorId>,
    default_surface: SurfaceId,
) -> Option<SurfaceId> {
    if !cut_connectors.contains(&connector_id) {
        return Some(default_surface);
    }
    universal
        .surface_cache
        .last_good
        .get(&target_surface)
        .copied()
        .or_else(|| universal.surface_cache.fallback.get(&target_surface).copied())
}

fn blend_state_for_mode(mode: u32) -> Option<wgpu::BlendState> {
    match mode {
        0 => Some(wgpu::BlendState::ALPHA_BLENDING),
        1 => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        _ => Some(wgpu::BlendState::ALPHA_BLENDING),
    }
}

fn find_realm_by_surface(universal: &UniversalState, surface: SurfaceId) -> Option<RealmId> {
    universal
        .realms
        .entries
        .iter()
        .find_map(|(realm_id, entry)| {
            if entry.value.output_surface == Some(surface) {
                Some(*realm_id)
            } else {
                None
            }
        })
}
