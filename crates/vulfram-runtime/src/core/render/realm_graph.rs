use std::collections::HashMap;

use crate::core::realm::{RealmId, SurfaceId, UniversalState};
use crate::core::resources::RenderTarget;
use crate::core::window::WindowState;

#[derive(Debug, Clone)]
pub(crate) struct SurfaceSnapshot {
    pub(crate) size: glam::UVec2,
}

pub(crate) fn map_realms_to_windows(universal: &UniversalState) -> HashMap<RealmId, u32> {
    let existing_realms = universal
        .composition
        .realms
        .entries
        .keys()
        .copied()
        .collect();
    let layer_windows: Vec<_> = universal
        .targets
        .target_layers
        .entries
        .values()
        .filter_map(|layer| {
            let target = universal.targets.targets.entries.get(&layer.target_id)?;
            Some((RealmId(layer.realm_id), target.window_id?))
        })
        .collect();
    let presents: Vec<_> = universal
        .composition
        .presents
        .entries
        .values()
        .map(|entry| (entry.value.surface, entry.value.window_id))
        .collect();
    let realm_output_surfaces: HashMap<_, _> = universal
        .composition
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
        .composition
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

pub(crate) fn collect_surface_views(
    device: &wgpu::Device,
    universal: &UniversalState,
    surface_targets: &mut HashMap<SurfaceId, RenderTarget>,
    present_sizes: &HashMap<SurfaceId, glam::UVec2>,
) -> HashMap<SurfaceId, SurfaceSnapshot> {
    surface_targets.retain(|surface_id, _| {
        universal
            .composition
            .surfaces
            .entries
            .contains_key(surface_id)
    });

    let mut views = HashMap::new();
    let surface_requests: Vec<_> = universal
        .composition
        .surfaces
        .entries
        .iter()
        .map(|(surface_id, entry)| vulfram_render::SurfaceTargetRequest {
            surface_id: *surface_id,
            declared_size: entry.value.size,
            is_onscreen: entry.value.kind == crate::core::realm::SurfaceKind::Onscreen,
        })
        .collect();
    let resolved_targets = vulfram_render::plan_surface_targets(&surface_requests, present_sizes);

    for resolved in resolved_targets {
        let Some(entry) = universal
            .composition
            .surfaces
            .entries
            .get(&resolved.surface_id)
        else {
            continue;
        };
        let target_format = entry
            .value
            .format_policy
            .unwrap_or(wgpu::TextureFormat::Rgba16Float);

        let _surface_target = ensure_surface_target(
            device,
            surface_targets,
            resolved.surface_id,
            resolved.target_size,
            target_format,
        );

        views.insert(
            resolved.surface_id,
            SurfaceSnapshot {
                size: resolved.target_size,
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
        .composition
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
    vulfram_render::ensure_surface_target(device, surface_targets, surface_id, size, format)
}
