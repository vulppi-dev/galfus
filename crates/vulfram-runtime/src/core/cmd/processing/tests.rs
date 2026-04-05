use super::super::*;
use super::defer::{DeferredFailureKind, classify_failed_response, command_signature};
use crate::core::state::EngineState;
use vulfram_runtime::defer_backoff_frames;

#[test]
fn defer_backoff_caps_at_64_frames() {
    assert_eq!(defer_backoff_frames(1), 1);
    assert_eq!(defer_backoff_frames(2), 2);
    assert_eq!(defer_backoff_frames(3), 4);
    assert_eq!(defer_backoff_frames(7), 64);
    assert_eq!(defer_backoff_frames(100), 64);
}

#[test]
fn command_signature_differs_for_different_payloads() {
    let cmd_a = EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
        window_id: 1,
        ..Default::default()
    });
    let cmd_b = EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
        window_id: 2,
        ..Default::default()
    });
    assert_ne!(command_signature(&cmd_a), command_signature(&cmd_b));
}

#[test]
fn classify_failed_response_marks_transient_dependency() {
    let engine = EngineState::new();
    let cmd = EngineCmd::CmdShadowConfigure(res::shadow::CmdShadowConfigureArgs {
        window_id: 10,
        config: res::shadow::ShadowConfig::default(),
    });
    let response = CommandResponse::ShadowConfigure(res::shadow::CmdResultShadowConfigure {
        success: false,
        message: "Window 10 not found or shadow manager not initialized".into(),
    });
    let classified = classify_failed_response(&engine, &cmd, &response);
    assert!(matches!(
        classified,
        Some((DeferredFailureKind::Transient, _))
    ));
}

#[test]
fn classify_failed_response_marks_permanent_invalid_payload() {
    let mut engine = EngineState::new();
    engine.buffers.insert_upload(
        77,
        crate::core::buffers::state::UploadBuffer {
            upload_type: crate::core::buffers::state::UploadType::Raw,
            data: vec![1, 2, 3],
        },
    );
    let cmd = EngineCmd::CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs {
        texture_id: 9,
        label: None,
        buffer_id: 77,
        srgb: Some(true),
        mode: res::TextureCreateMode::Standalone,
        atlas_options: None,
    });
    let response =
        CommandResponse::TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer {
            success: false,
            message: "Invalid buffer type. Expected ImageData, got Raw".into(),
            pending: false,
        });
    let classified = classify_failed_response(&engine, &cmd, &response);
    assert!(matches!(
        classified,
        Some((DeferredFailureKind::Permanent, _))
    ));
}
