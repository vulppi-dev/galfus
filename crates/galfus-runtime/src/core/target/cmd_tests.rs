use super::{
    CmdTargetDisposeArgs, CmdTargetGetArgs, CmdTargetLayerDisposeArgs, CmdTargetLayerGetArgs,
    CmdTargetLayerListArgs, CmdTargetLayerUpsertArgs, CmdTargetListArgs, CmdTargetMeasurementArgs,
    CmdTargetUpsertArgs, engine_cmd_target_dispose, engine_cmd_target_get,
    engine_cmd_target_layer_dispose, engine_cmd_target_layer_get, engine_cmd_target_layer_list,
    engine_cmd_target_layer_upsert, engine_cmd_target_list, engine_cmd_target_measurement,
    engine_cmd_target_upsert,
};
use crate::core::target::{DimensionValue, TargetLayerLayout};
use crate::core::target::{TargetId, TargetKind};
use crate::core::test_support::{alloc_offscreen_surface, link_target_surface, test_engine};
use glam::UVec2;

#[test]
fn target_upsert_window_requires_window_id() {
    let mut engine = test_engine();
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
fn target_upsert_texture_allows_missing_window_id() {
    let mut engine = test_engine();
    let result = engine_cmd_target_upsert(
        &mut engine,
        &CmdTargetUpsertArgs {
            target_id: 2,
            kind: TargetKind::Texture,
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
fn target_upsert_texture_allows_missing_window_id_second_target() {
    let mut engine = test_engine();
    let result = engine_cmd_target_upsert(
        &mut engine,
        &CmdTargetUpsertArgs {
            target_id: 3,
            kind: TargetKind::Texture,
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
    let mut engine = test_engine();
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
    let mut engine = test_engine();
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
    let mut engine = test_engine();
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
    let surface_id = alloc_offscreen_surface(&mut engine, UVec2::new(640, 360));
    link_target_surface(
        &mut engine,
        crate::core::realm::RealmId(7),
        TargetId(51),
        surface_id,
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

#[test]
fn target_get_and_list_respect_filters() {
    let mut engine = test_engine();
    assert!(
        engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 70,
                kind: TargetKind::Window,
                window_id: Some(1),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        )
        .success
    );
    assert!(
        engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 71,
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(32, 32)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        )
        .success
    );

    let get = engine_cmd_target_get(&mut engine, &CmdTargetGetArgs { target_id: 70 });
    assert!(get.success);
    assert_eq!(get.window_id, Some(1));

    let listed = engine_cmd_target_list(
        &mut engine,
        &CmdTargetListArgs {
            window_id: Some(1),
            ids: None,
        },
    );
    assert!(listed.success);
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].target_id, 70);
}

#[test]
fn target_layer_get_list_and_dispose_work() {
    let mut engine = test_engine();
    assert!(
        engine_cmd_target_upsert(
            &mut engine,
            &CmdTargetUpsertArgs {
                target_id: 80,
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(64, 64)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        )
        .success
    );

    let layout = TargetLayerLayout {
        left: DimensionValue::Px(0.0),
        top: DimensionValue::Px(0.0),
        width: DimensionValue::Px(64.0),
        height: DimensionValue::Px(64.0),
        enabled: true,
        opacity: 1.0,
        z_index: 0,
        blend_mode: 0,
        clip: None,
    };
    let upsert = engine_cmd_target_layer_upsert(
        &mut engine,
        &CmdTargetLayerUpsertArgs {
            realm_id: 9,
            target_id: 80,
            layout,
            enabled_camera_ids: vec![5, 6],
            environment_id: Some(7),
        },
    );
    assert!(upsert.success);

    let get = engine_cmd_target_layer_get(
        &mut engine,
        &CmdTargetLayerGetArgs {
            realm_id: 9,
            target_id: 80,
        },
    );
    assert!(get.success);
    assert_eq!(get.enabled_camera_ids, vec![5, 6]);
    assert_eq!(get.environment_id, Some(7));

    let listed = engine_cmd_target_layer_list(
        &mut engine,
        &CmdTargetLayerListArgs {
            realm_id: Some(9),
            target_id: Some(80),
        },
    );
    assert!(listed.success);
    assert_eq!(listed.items.len(), 1);

    let disposed_layer = engine_cmd_target_layer_dispose(
        &mut engine,
        &CmdTargetLayerDisposeArgs {
            realm_id: 9,
            target_id: 80,
        },
    );
    assert!(disposed_layer.success);

    let disposed_target =
        engine_cmd_target_dispose(&mut engine, &CmdTargetDisposeArgs { target_id: 80 });
    assert!(disposed_target.success);
}
