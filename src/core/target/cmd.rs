use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::target::resolve::remove_auto_link_for_layer;
use crate::core::target::{
    SurfaceAlphaModeDto, SurfaceFormatDto, TargetId, TargetKind, TargetLayerLayout,
    TargetLayerState, TargetState,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetUpsertArgs {
    pub target_id: u64,
    pub kind: TargetKind,
    #[serde(default)]
    pub window_id: Option<u32>,
    #[serde(default)]
    pub size: Option<UVec2>,
    #[serde(default)]
    pub format_policy: Option<SurfaceFormatDto>,
    #[serde(default)]
    pub alpha_policy: Option<SurfaceAlphaModeDto>,
    #[serde(default)]
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetMeasurementArgs {
    pub target_id: u64,
    pub get_size: bool,
    pub get_window_size: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetMeasurement {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub size: Option<UVec2>,
    #[serde(default)]
    pub window_size: Option<UVec2>,
    #[serde(default)]
    pub source_kind: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetDisposeArgs {
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetLayerUpsertArgs {
    pub realm_id: u32,
    pub target_id: u64,
    pub layout: TargetLayerLayout,
    #[serde(default)]
    pub camera_id: Option<u32>,
    #[serde(default)]
    pub environment_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetLayerDisposeArgs {
    pub realm_id: u32,
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_target_upsert(
    engine: &mut EngineState,
    args: &CmdTargetUpsertArgs,
) -> CmdResultTargetUpsert {
    if matches!(args.kind, TargetKind::Window) && args.window_id.is_none() {
        return CmdResultTargetUpsert {
            success: false,
            message: format!("Target {:?} requires windowId", args.kind),
        };
    }
    if matches!(args.kind, TargetKind::Texture) && args.window_id.is_some() {
        return CmdResultTargetUpsert {
            success: false,
            message: "Target texture does not accept windowId".into(),
        };
    }
    if !matches!(args.kind, TargetKind::Texture) && args.size.is_some() {
        return CmdResultTargetUpsert {
            success: false,
            message: "Target size is only valid for kind=texture".into(),
        };
    }
    let size = args
        .size
        .map(|size| UVec2::new(size.x.max(1), size.y.max(1)));
    engine.universal_state.targets.entries.insert(
        TargetId(args.target_id),
        TargetState {
            kind: args.kind,
            window_id: args.window_id,
            size,
            format_policy: args.format_policy.map(SurfaceFormatDto::to_wgpu),
            alpha_policy: args.alpha_policy.map(SurfaceAlphaModeDto::to_wgpu),
            msaa_samples: args.msaa_samples,
        },
    );

    CmdResultTargetUpsert {
        success: true,
        message: "Target upserted".into(),
    }
}

pub fn engine_cmd_target_measurement(
    engine: &mut EngineState,
    args: &CmdTargetMeasurementArgs,
) -> CmdResultTargetMeasurement {
    let target_id = TargetId(args.target_id);
    let Some(target) = engine.universal_state.targets.entries.get(&target_id) else {
        return CmdResultTargetMeasurement {
            success: true,
            message: format!(
                "Target {} not ready yet; returning empty measurement",
                args.target_id
            ),
            ..Default::default()
        };
    };

    let resolved = resolve_target_measurement_size(engine, target_id);
    let window_size = if args.get_window_size {
        target
            .window_id
            .and_then(|window_id| engine.window.states.get(&window_id))
            .map(|state| state.inner_size)
    } else {
        None
    };
    let size = if args.get_size || !args.get_window_size {
        resolved.map(|(size, _)| size)
    } else {
        None
    };
    let source_kind = if args.get_size || !args.get_window_size {
        resolved.map(|(_, source)| source.to_string())
    } else {
        None
    };

    CmdResultTargetMeasurement {
        success: true,
        message: "Target measurement command applied successfully".into(),
        size,
        window_size,
        source_kind,
    }
}

fn resolve_target_measurement_size(
    engine: &EngineState,
    target_id: TargetId,
) -> Option<(UVec2, &'static str)> {
    let target = engine.universal_state.targets.entries.get(&target_id)?;

    let surface_id = engine
        .universal_state
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
        .map(|(_, surface_id)| surface_id);
    if let Some(surface_id) = surface_id
        && let Some(surface) = engine.universal_state.surfaces.entries.get(&surface_id)
    {
        return Some((surface.value.size, "surface"));
    }

    if let Some(window_id) = target.window_id
        && let Some(window_state) = engine.window.states.get(&window_id)
    {
        return Some((
            UVec2::new(
                window_state.config.width.max(1),
                window_state.config.height.max(1),
            ),
            "window-surface",
        ));
    }

    target.size.map(|size| (size, "declared"))
}

pub fn engine_cmd_target_dispose(
    engine: &mut EngineState,
    args: &CmdTargetDisposeArgs,
) -> CmdResultTargetDispose {
    let target_id = TargetId(args.target_id);
    if engine
        .universal_state
        .targets
        .entries
        .remove(&target_id)
        .is_none()
    {
        return CmdResultTargetDispose {
            success: false,
            message: format!("Target {} not found", args.target_id),
        };
    }

    engine
        .universal_state
        .target_layers
        .entries
        .retain(|(_, layer_target), _| *layer_target != target_id);

    let remove_keys: Vec<_> = engine
        .universal_state
        .auto_links
        .keys()
        .filter(|(_, layer_target)| *layer_target == target_id)
        .copied()
        .collect();
    for (realm_id, layer_target) in remove_keys {
        remove_auto_link_for_layer(&mut engine.universal_state, realm_id, layer_target);
    }

    engine
        .universal_state
        .input_routing
        .focus_targets
        .retain(|_, focus_target_id| *focus_target_id != target_id);
    engine
        .universal_state
        .ui
        .external_textures
        .remove(&target_id.0);
    engine
        .universal_state
        .ui
        .target_size_requests
        .remove(&target_id.0);
    engine
        .universal_state
        .target_ui_realm_index
        .remove(&target_id);
    let _ = engine
        .universal_state
        .target_listeners
        .dispose_target(target_id);
    engine
        .universal_state
        .global_resources
        .target_texture_binds
        .retain(|_, binding| binding.target_id != target_id);
    engine
        .universal_state
        .target_graph_cache
        .prune_dead_entries(
            &engine.universal_state.targets.entries,
            &engine.universal_state.target_layers.entries,
            &engine.universal_state.realms,
        );

    CmdResultTargetDispose {
        success: true,
        message: "Target disposed".into(),
    }
}

pub fn engine_cmd_target_layer_upsert(
    engine: &mut EngineState,
    args: &CmdTargetLayerUpsertArgs,
) -> CmdResultTargetLayerUpsert {
    let target_id = TargetId(args.target_id);
    if args.layout.width.resolve(1.0, 8.0) <= 0.0 || args.layout.height.resolve(1.0, 8.0) <= 0.0 {
        return CmdResultTargetLayerUpsert {
            success: false,
            message: "TargetLayer layout width/height must be > 0".into(),
        };
    }
    engine.universal_state.target_layers.entries.insert(
        (args.realm_id, target_id),
        TargetLayerState {
            realm_id: args.realm_id,
            target_id,
            layout: args.layout,
            camera_id: args.camera_id,
            environment_id: args.environment_id,
        },
    );

    CmdResultTargetLayerUpsert {
        success: true,
        message: "TargetLayer upserted".into(),
    }
}

pub fn engine_cmd_target_layer_dispose(
    engine: &mut EngineState,
    args: &CmdTargetLayerDisposeArgs,
) -> CmdResultTargetLayerDispose {
    let target_id = TargetId(args.target_id);
    if engine
        .universal_state
        .target_layers
        .entries
        .remove(&(args.realm_id, target_id))
        .is_none()
    {
        return CmdResultTargetLayerDispose {
            success: false,
            message: format!(
                "Layer not found (realm_id={}, target_id={})",
                args.realm_id, args.target_id
            ),
        };
    }

    remove_auto_link_for_layer(&mut engine.universal_state, args.realm_id, target_id);
    let has_layer_for_target = engine
        .universal_state
        .target_layers
        .entries
        .keys()
        .any(|(_, layer_target)| *layer_target == target_id);
    if !has_layer_for_target {
        engine
            .universal_state
            .input_routing
            .focus_targets
            .retain(|_, focus_target_id| *focus_target_id != target_id);
    }
    engine
        .universal_state
        .target_graph_cache
        .prune_dead_entries(
            &engine.universal_state.targets.entries,
            &engine.universal_state.target_layers.entries,
            &engine.universal_state.realms,
        );

    CmdResultTargetLayerDispose {
        success: true,
        message: "TargetLayer disposed".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CmdTargetMeasurementArgs, CmdTargetUpsertArgs, engine_cmd_target_measurement,
        engine_cmd_target_upsert,
    };
    use crate::core::realm::{AutoLink, SurfaceKind, SurfaceState};
    use crate::core::state::EngineState;
    use crate::core::target::{TargetId, TargetKind};
    use glam::UVec2;

    #[test]
    fn target_upsert_window_requires_window_id() {
        let mut engine = EngineState::new();
        let result = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 1,
                kind: TargetKind::Window,
                window_id: None,
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(!result.success);
        assert!(result.message.contains("requires windowId"));
    }

    #[test]
    fn target_upsert_widget_viewport_allows_missing_window_id() {
        let mut engine = EngineState::new();
        let result = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 2,
                kind: TargetKind::WidgetRealmViewport,
                window_id: None,
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(result.success);
    }

    #[test]
    fn target_upsert_realm_plane_allows_missing_window_id() {
        let mut engine = EngineState::new();
        let result = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 3,
                kind: TargetKind::RealmPlane,
                window_id: None,
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(result.success);
    }

    #[test]
    fn target_upsert_texture_rejects_window_id() {
        let mut engine = EngineState::new();
        let result = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 4,
                kind: TargetKind::Texture,
                window_id: Some(10),
                size: Some(UVec2::new(128, 128)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(!result.success);
        assert!(result.message.contains("does not accept windowId"));
    }

    #[test]
    fn target_measurement_uses_declared_size_when_no_runtime_binding_exists() {
        let mut engine = EngineState::new();
        let upsert = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 50,
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(256, 128)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(upsert.success);

        let measured = engine_cmd_target_measurement(
            &mut engine,
            &CmdTargetMeasurementArgs {
                target_id: 50,
                get_size: true,
                get_window_size: false,
            },
        );
        assert!(measured.success);
        assert_eq!(measured.size, Some(UVec2::new(256, 128)));
        assert_eq!(measured.source_kind.as_deref(), Some("declared"));
    }

    #[test]
    fn target_measurement_prefers_surface_size_from_auto_link() {
        let mut engine = EngineState::new();
        let upsert = engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 51,
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(16, 16)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        assert!(upsert.success);
        let surface_id = engine.universal_state.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: UVec2::new(640, 360),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        engine.universal_state.auto_links.insert(
            (7, TargetId(51)),
            AutoLink {
                surface_id,
                connector_id: None,
                present_id: None,
            },
        );

        let measured = engine_cmd_target_measurement(
            &mut engine,
            &CmdTargetMeasurementArgs {
                target_id: 51,
                get_size: true,
                get_window_size: false,
            },
        );
        assert!(measured.success);
        assert_eq!(measured.size, Some(UVec2::new(640, 360)));
        assert_eq!(measured.source_kind.as_deref(), Some("surface"));
    }
}
