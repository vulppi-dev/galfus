use std::collections::HashMap;

use vulfram_realm_core::{SurfaceId, TargetId, TargetKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceTargetRequest {
    pub surface_id: SurfaceId,
    pub declared_size: glam::UVec2,
    pub is_onscreen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedSurfaceTarget {
    pub surface_id: SurfaceId,
    pub target_size: glam::UVec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSizeUpdateRequest {
    pub target_id: TargetId,
    pub kind: TargetKind,
    pub current_size: Option<glam::UVec2>,
    pub requested_size: glam::UVec2,
    pub msaa_samples: Option<u32>,
    pub window_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSizeUpdatePlanEntry {
    pub target_id: TargetId,
    pub desired_size: glam::UVec2,
    pub needs_size_update: bool,
    pub needs_msaa_init: bool,
    pub window_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalTextureSource {
    pub texture_id: u32,
    pub source_key: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalTextureRefreshPlan {
    pub stale_ids: Vec<u32>,
    pub replace_ids: Vec<u32>,
}

pub fn plan_surface_targets(
    requests: &[SurfaceTargetRequest],
    present_sizes: &HashMap<SurfaceId, glam::UVec2>,
) -> Vec<ResolvedSurfaceTarget> {
    requests
        .iter()
        .map(|request| {
            let target_size = if request.is_onscreen {
                present_sizes
                    .get(&request.surface_id)
                    .copied()
                    .unwrap_or(request.declared_size)
            } else {
                request.declared_size
            };
            ResolvedSurfaceTarget {
                surface_id: request.surface_id,
                target_size,
            }
        })
        .collect()
}

pub fn plan_target_size_updates(
    requests: &[TargetSizeUpdateRequest],
) -> Vec<TargetSizeUpdatePlanEntry> {
    requests
        .iter()
        .filter(|request| request.kind != TargetKind::Window)
        .map(|request| {
            let desired_size = glam::UVec2::new(
                request.requested_size.x.max(1),
                request.requested_size.y.max(1),
            );
            TargetSizeUpdatePlanEntry {
                target_id: request.target_id,
                desired_size,
                needs_size_update: request.current_size != Some(desired_size),
                needs_msaa_init: request.msaa_samples.is_none(),
                window_id: request.window_id,
            }
        })
        .collect()
}

pub fn plan_external_texture_refresh(
    current_sources: &HashMap<u32, usize>,
    next_sources: &[ExternalTextureSource],
) -> ExternalTextureRefreshPlan {
    let next_by_id: HashMap<_, _> = next_sources
        .iter()
        .map(|source| (source.texture_id, source.source_key))
        .collect();

    let mut stale_ids: Vec<u32> = current_sources
        .keys()
        .filter(|texture_id| !next_by_id.contains_key(texture_id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next_sources
        .iter()
        .filter(|source| {
            current_sources.get(&source.texture_id).copied() != Some(source.source_key)
        })
        .map(|source| source.texture_id)
        .collect();
    replace_ids.sort_unstable();

    ExternalTextureRefreshPlan {
        stale_ids,
        replace_ids,
    }
}
