use super::{
    CmdTargetMeasurementArgs, CmdTargetUpsertArgs, engine_cmd_target_measurement,
    engine_cmd_target_upsert,
};
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
fn target_upsert_widget_viewport_allows_missing_window_id() {
    let mut engine = test_engine();
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
    let mut engine = test_engine();
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
