use std::collections::{HashMap, HashSet};

use galfus_realm_core::{ConnectorId, RealmId, SurfaceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeBlendMode {
    Alpha,
    PremultipliedAlpha,
    Replace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComposeConnectorCandidate {
    pub connector_id: ConnectorId,
    pub source_surface: SurfaceId,
    pub rect: glam::Vec4,
    pub clip: Option<glam::Vec4>,
    pub z_index: i32,
    pub blend_mode: u32,
    pub widget_view: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComposeOverlayPlanEntry {
    pub connector_id: ConnectorId,
    pub source_surface: SurfaceId,
    pub rect: glam::Vec4,
    pub clip: Option<glam::Vec4>,
    pub z_index: i32,
    pub blend_mode: ComposeBlendMode,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComposeOverlayPlan {
    pub blocked_connectors: Vec<ConnectorId>,
    pub self_sampled_connectors: Vec<ConnectorId>,
    pub no_progress_realms: Vec<RealmId>,
    pub overlays: Vec<ComposeOverlayPlanEntry>,
}

pub fn resolve_connector_surface(
    cut_connectors: &HashSet<ConnectorId>,
    last_good: &HashMap<ConnectorId, SurfaceId>,
    fallback: &HashMap<ConnectorId, SurfaceId>,
    connector_id: ConnectorId,
    default_surface: SurfaceId,
) -> SurfaceId {
    if !cut_connectors.contains(&connector_id) {
        return default_surface;
    }

    last_good
        .get(&connector_id)
        .copied()
        .or_else(|| fallback.get(&connector_id).copied())
        .unwrap_or(default_surface)
}

fn push_unique_id(list: &mut Vec<u32>, value: u32) {
    if !list.contains(&value) {
        list.push(value);
    }
}

pub fn plan_compose_overlays(
    connector_candidates: &[ComposeConnectorCandidate],
    target_surface: SurfaceId,
    cut_connectors: &HashSet<ConnectorId>,
    last_good: &HashMap<ConnectorId, SurfaceId>,
    fallback: &HashMap<ConnectorId, SurfaceId>,
    available_surfaces: &HashSet<SurfaceId>,
    realm_id: RealmId,
) -> ComposeOverlayPlan {
    let mut blocked_connectors = Vec::new();
    let mut self_sampled_connectors = Vec::new();
    let mut no_progress_realms = Vec::new();
    let mut overlays = Vec::new();

    for candidate in connector_candidates {
        if candidate.widget_view {
            continue;
        }
        if candidate.source_surface == target_surface {
            push_unique_id(&mut self_sampled_connectors, candidate.connector_id.0);
            push_unique_id(&mut no_progress_realms, realm_id.0);
            continue;
        }

        let source_surface = resolve_connector_surface(
            cut_connectors,
            last_good,
            fallback,
            candidate.connector_id,
            candidate.source_surface,
        );
        if !available_surfaces.contains(&source_surface) {
            push_unique_id(&mut blocked_connectors, candidate.connector_id.0);
            push_unique_id(&mut no_progress_realms, realm_id.0);
            continue;
        }

        overlays.push(ComposeOverlayPlanEntry {
            connector_id: candidate.connector_id,
            source_surface,
            rect: candidate.rect,
            clip: candidate.clip,
            z_index: candidate.z_index,
            blend_mode: match candidate.blend_mode {
                1 => ComposeBlendMode::PremultipliedAlpha,
                2 => ComposeBlendMode::Replace,
                _ => ComposeBlendMode::Alpha,
            },
        });
    }

    overlays.sort_by_key(|entry| entry.z_index);

    ComposeOverlayPlan {
        blocked_connectors: blocked_connectors.into_iter().map(ConnectorId).collect(),
        self_sampled_connectors: self_sampled_connectors
            .into_iter()
            .map(ConnectorId)
            .collect(),
        no_progress_realms: no_progress_realms.into_iter().map(RealmId).collect(),
        overlays,
    }
}
