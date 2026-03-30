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
                .entries
                .get(&target_id)
                .and_then(|target| target.size)
        }),
    })
}

fn connector_source_size(universal: &UniversalState, connector_id: ConnectorId) -> Option<UVec2> {
    let connector = universal.connectors.entries.get(&connector_id)?;
    universal
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
        .surfaces
        .entries
        .get(&surface_id)
        .map(|entry| entry.value.size)
}

#[cfg(test)]
mod tests {
    use super::resolve_target_size;
    use crate::core::realm::{AutoLink, RealmId, SurfaceKind, SurfaceState};
    use crate::core::state::EngineState;
    use crate::core::target::{TargetId, TargetKind, TargetState};
    use glam::UVec2;

    #[test]
    fn resolve_target_size_prefers_target_surface_over_declared_size() {
        let mut engine = EngineState::new();
        let target_id = TargetId(700);
        engine.universal_state.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(300, 200)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );

        let surface_id = engine.universal_state.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: UVec2::new(1280, 720),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        engine.universal_state.auto_links.insert(
            (RealmId(1).0, target_id),
            AutoLink {
                surface_id,
                connector_id: None,
                present_id: None,
            },
        );

        let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
        assert_eq!(size, Some(UVec2::new(1280, 720)));
    }

    #[test]
    fn resolve_target_size_falls_back_to_declared_without_runtime_surface() {
        let mut engine = EngineState::new();
        let target_id = TargetId(701);
        engine.universal_state.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(640, 360)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );

        let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
        assert_eq!(size, Some(UVec2::new(640, 360)));
    }
}
