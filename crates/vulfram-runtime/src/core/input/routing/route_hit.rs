use glam::{UVec2, Vec2};

use crate::core::realm::{ConnectorId, ConnectorState, RealmId, UniversalState};

use super::route_cache::realm_surface_size;
pub(super) use vulfram_input::{InputTargetSizing, resolve_hit_connector};

pub(super) fn resolve_connector_uv(
    universal: &UniversalState,
    connector: &ConnectorState,
    position: Vec2,
    target_size: UVec2,
) -> Option<Vec2> {
    let source_size = universal
        .composition
        .surfaces
        .entries
        .get(&connector.source_surface)
        .map(|entry| entry.value.size)?;
    vulfram_input::resolve_connector_uv_from_sizes(
        connector.rect,
        connector.clip,
        position,
        source_size,
        target_size,
    )
}

pub(super) fn resolve_target_relative_position(
    universal: &UniversalState,
    source_realm_id: Option<RealmId>,
    connector_id: Option<ConnectorId>,
    uv: Option<Vec2>,
) -> Option<Vec2> {
    vulfram_input::resolve_target_relative_position(
        InputTargetSizing {
            source_realm_size: source_realm_id
                .and_then(|realm_id| realm_surface_size(universal, realm_id)),
            connector_source_size: connector_id
                .and_then(|connector_id| connector_source_size(universal, connector_id)),
            target_surface_size: None,
            target_declared_size: None,
        },
        uv,
    )
}

pub(super) fn resolve_target_size(
    universal: &UniversalState,
    source_realm_id: Option<RealmId>,
    connector_id: Option<ConnectorId>,
    target_id: Option<crate::core::target::TargetId>,
) -> Option<UVec2> {
    vulfram_input::resolve_target_size(InputTargetSizing {
        source_realm_size: source_realm_id
            .and_then(|realm_id| realm_surface_size(universal, realm_id)),
        connector_source_size: connector_id
            .and_then(|connector_id| connector_source_size(universal, connector_id)),
        target_surface_size: target_id
            .and_then(|target_id| target_surface_size(universal, target_id)),
        target_declared_size: target_id.and_then(|target_id| {
            universal
                .targets
                .targets
                .entries
                .get(&target_id)
                .and_then(|target| target.size)
        }),
    })
}

fn connector_source_size(universal: &UniversalState, connector_id: ConnectorId) -> Option<UVec2> {
    let connector = universal
        .composition
        .connectors
        .entries
        .get(&connector_id)?;
    universal
        .composition
        .surfaces
        .entries
        .get(&connector.value.source_surface)
        .map(|entry| entry.value.size)
}

fn target_surface_size(
    universal: &UniversalState,
    target_id: crate::core::target::TargetId,
) -> Option<UVec2> {
    let surface_id = universal
        .targets
        .auto_links
        .iter()
        .filter_map(|((realm_id, layer_target_id), link)| {
            if *layer_target_id == target_id {
                Some((*realm_id, link.surface_id))
            } else {
                None
            }
        })
        .min_by_key(|(realm_id, _)| *realm_id)
        .map(|(_, surface_id)| surface_id)?;

    universal
        .composition
        .surfaces
        .entries
        .get(&surface_id)
        .map(|entry| entry.value.size)
}

#[cfg(test)]
#[path = "route_hit_tests.rs"]
mod tests;
