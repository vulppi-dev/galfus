use glam::{UVec2, Vec2};

use crate::core::realm::{
    ConnectorId, ConnectorState, InputRoutingConnectorHit, RealmId, UniversalState,
};

use super::route_cache::realm_surface_size;

const INPUT_FLAG_RAYCAST: u32 = 1 << 0;

#[derive(Debug, Clone, Copy)]
pub(super) struct HitResult {
    pub(super) connector_id: ConnectorId,
    pub(super) uv: Option<Vec2>,
}

pub(super) fn resolve_hit_connector(
    connectors: Option<&Vec<InputRoutingConnectorHit>>,
    position: Vec2,
    window_size: Option<UVec2>,
) -> Option<HitResult> {
    let connectors = connectors?;
    let target_size = window_size.unwrap_or_else(|| UVec2::new(1, 1));
    for connector in connectors {
        if connector.state.input_flags & INPUT_FLAG_RAYCAST != 0 {
            if hit_test_connector(
                position,
                connector.state.rect,
                connector.state.clip,
                connector.source_size,
                target_size,
            ) {
                let uv = resolve_connector_uv_from_sizes(
                    connector.state.rect,
                    connector.state.clip,
                    position,
                    connector.source_size,
                    target_size,
                );
                return Some(HitResult {
                    connector_id: connector.id,
                    uv,
                });
            }
            continue;
        }
        if hit_test_connector(
            position,
            connector.state.rect,
            connector.state.clip,
            connector.source_size,
            target_size,
        ) {
            return Some(HitResult {
                connector_id: connector.id,
                uv: None,
            });
        }
    }
    None
}

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
    resolve_connector_uv_from_sizes(
        connector.rect,
        connector.clip,
        position,
        source_size,
        target_size,
    )
}

fn resolve_connector_uv_from_sizes(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    position: Vec2,
    source_size: UVec2,
    target_size: UVec2,
) -> Option<Vec2> {
    let (viewport, _) = resolve_overlay_geometry(rect, clip, source_size, target_size)?;
    let u = ((position.x - viewport.x) / viewport.z.max(1.0)).clamp(0.0, 1.0);
    let v = ((position.y - viewport.y) / viewport.w.max(1.0)).clamp(0.0, 1.0);
    Some(Vec2::new(u, v))
}

pub(super) fn resolve_target_relative_position(
    universal: &UniversalState,
    source_realm_id: Option<RealmId>,
    connector_id: Option<ConnectorId>,
    uv: Option<Vec2>,
) -> Option<Vec2> {
    let uv = uv?;
    let size = source_realm_id
        .and_then(|realm_id| realm_surface_size(universal, realm_id))
        .or_else(|| {
            connector_id.and_then(|connector_id| connector_source_size(universal, connector_id))
        })?;
    Some(Vec2::new(
        uv.x.clamp(0.0, 1.0) * size.x.max(1) as f32,
        uv.y.clamp(0.0, 1.0) * size.y.max(1) as f32,
    ))
}

pub(super) fn resolve_target_size(
    universal: &UniversalState,
    source_realm_id: Option<RealmId>,
    connector_id: Option<ConnectorId>,
    target_id: Option<crate::core::target::TargetId>,
) -> Option<UVec2> {
    source_realm_id
        .and_then(|realm_id| realm_surface_size(universal, realm_id))
        .or_else(|| {
            connector_id.and_then(|connector_id| connector_source_size(universal, connector_id))
        })
        .or_else(|| target_id.and_then(|target_id| target_surface_size(universal, target_id)))
        .or_else(|| {
            target_id.and_then(|target_id| {
                universal
                    .targets
                    .entries
                    .get(&target_id)
                    .and_then(|target| target.size)
            })
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

fn hit_test_connector(
    position: Vec2,
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: UVec2,
) -> bool {
    let Some((viewport, clip_rect)) =
        resolve_overlay_geometry(rect, clip, source_size, target_size)
    else {
        return false;
    };

    let inside_viewport = position.x >= viewport.x
        && position.y >= viewport.y
        && position.x <= viewport.x + viewport.z
        && position.y <= viewport.y + viewport.w;
    let inside_clip = position.x >= clip_rect.x
        && position.y >= clip_rect.y
        && position.x <= clip_rect.x + clip_rect.z
        && position.y <= clip_rect.y + clip_rect.w;
    inside_viewport && inside_clip
}

fn resolve_overlay_geometry(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: UVec2,
) -> Option<(glam::Vec4, glam::Vec4)> {
    if rect.z <= 0.0 || rect.w <= 0.0 {
        return None;
    }

    let source_width = source_size.x.max(1) as f32;
    let source_height = source_size.y.max(1) as f32;
    let scale = rect.w / source_height;
    let draw_width = (source_width * scale).max(1.0);

    let mut viewport_x = rect.x + (rect.z - draw_width) * 0.5;
    let mut viewport_y = rect.y;
    let mut viewport_width = draw_width;
    let mut viewport_height = rect.w.max(1.0);

    if viewport_x < 0.0 {
        viewport_width = (viewport_width + viewport_x).max(0.0);
        viewport_x = 0.0;
    }
    if viewport_y < 0.0 {
        viewport_height = (viewport_height + viewport_y).max(0.0);
        viewport_y = 0.0;
    }

    let max_width = target_size.x as f32 - viewport_x;
    let max_height = target_size.y as f32 - viewport_y;
    if max_width <= 0.0 || max_height <= 0.0 {
        return None;
    }
    viewport_width = viewport_width.min(max_width);
    viewport_height = viewport_height.min(max_height);
    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return None;
    }

    let viewport = glam::Vec4::new(viewport_x, viewport_y, viewport_width, viewport_height);
    let mut clip_rect = rect;
    if let Some(clip) = clip {
        clip_rect = intersect_rect(clip_rect, clip);
    }
    Some((viewport, clip_rect))
}

fn intersect_rect(a: glam::Vec4, b: glam::Vec4) -> glam::Vec4 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.z).min(b.x + b.z);
    let y2 = (a.y + a.w).min(b.y + b.w);
    glam::Vec4::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
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
