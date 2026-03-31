use super::super::*;
use crate::core::state::EngineState;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use vulfram_runtime::DeferredCommandKey;

#[derive(PartialEq, Eq)]
pub(super) enum DeferredFailureKind {
    Transient,
    Permanent,
}

pub(super) use vulfram_runtime::defer_backoff_frames;

pub(super) fn command_signature(cmd: &EngineCmd) -> u64 {
    let mut hasher = DefaultHasher::new();
    match rmp_serde::to_vec_named(cmd) {
        Ok(bytes) => hasher.write(&bytes),
        Err(_) => {
            hasher.write_u8(command_type_for_cmd(cmd).len() as u8);
            hasher.write(command_type_for_cmd(cmd).as_bytes());
        }
    }
    hasher.finish()
}

pub(crate) fn deferred_command_key(command_id: u64, cmd: &EngineCmd) -> DeferredCommandKey {
    DeferredCommandKey {
        command_id,
        command_signature: command_signature(cmd),
    }
}

pub(super) fn command_type_for_cmd(cmd: &EngineCmd) -> &'static str {
    match cmd {
        EngineCmd::CmdNotificationSend(_) => "notification-send",
        EngineCmd::CmdSystemDiagnosticsSet(_) => "system-diagnostics-set",
        EngineCmd::CmdSystemBuildVersionGet(_) => "system-build-version-get",
        EngineCmd::CmdWindowCreate(_) => "window-create",
        EngineCmd::CmdWindowClose(_) => "window-close",
        EngineCmd::CmdWindowMeasurement(_) => "window-measurement",
        EngineCmd::CmdWindowCursor(_) => "window-cursor",
        EngineCmd::CmdWindowState(_) => "window-state",
        EngineCmd::CmdInputTargetListenerUpsert(_) => "input-target-listener-upsert",
        EngineCmd::CmdInputTargetListenerDispose(_) => "input-target-listener-dispose",
        EngineCmd::CmdInputTargetListenerList(_) => "input-target-listener-list",
        EngineCmd::CmdUploadBufferDiscardAll(_) => "upload-buffer-discard-all",
        EngineCmd::CmdCameraUpsert(_) => "camera-upsert",
        EngineCmd::CmdCameraDispose(_) => "camera-dispose",
        EngineCmd::CmdModelUpsert(_) => "model-upsert",
        EngineCmd::CmdPoseUpdate(_) => "pose-update",
        EngineCmd::CmdModelDispose(_) => "model-dispose",
        EngineCmd::CmdLightUpsert(_) => "light-upsert",
        EngineCmd::CmdLightDispose(_) => "light-dispose",
        EngineCmd::CmdMaterialUpsert(_) => "material-upsert",
        EngineCmd::CmdMaterialDispose(_) => "material-dispose",
        EngineCmd::CmdTextureCreateFromBuffer(_) => "texture-create-from-buffer",
        EngineCmd::CmdTextureCreateSolidColor(_) => "texture-create-solid-color",
        EngineCmd::CmdTextureDispose(_) => "texture-dispose",
        EngineCmd::CmdTextureBindTarget(_) => "texture-bind-target",
        EngineCmd::CmdAudioListenerUpsert(_) => "audio-listener-upsert",
        EngineCmd::CmdAudioListenerDispose(_) => "audio-listener-dispose",
        EngineCmd::CmdAudioResourceUpsert(_) => "audio-resource-upsert",
        EngineCmd::CmdAudioSourceUpsert(_) => "audio-source-upsert",
        EngineCmd::CmdAudioSourceTransport(_) => "audio-source-transport",
        EngineCmd::CmdAudioStateGet(_) => "audio-state-get",
        EngineCmd::CmdAudioSourceDispose(_) => "audio-source-dispose",
        EngineCmd::CmdAudioResourceDispose(_) => "audio-resource-dispose",
        EngineCmd::CmdGeometryUpsert(_) => "geometry-upsert",
        EngineCmd::CmdGeometryDispose(_) => "geometry-dispose",
        EngineCmd::CmdPrimitiveGeometryCreate(_) => "primitive-geometry-create",
        EngineCmd::CmdEnvironmentUpsert(_) => "environment-upsert",
        EngineCmd::CmdEnvironmentDispose(_) => "environment-dispose",
        EngineCmd::CmdShadowConfigure(_) => "shadow-configure",
        EngineCmd::CmdRealmCreate(_) => "realm-create",
        EngineCmd::CmdRealmDispose(_) => "realm-dispose",
        EngineCmd::CmdRenderGraphUpsert(_) => "render-graph-upsert",
        EngineCmd::CmdRenderGraphDispose(_) => "render-graph-dispose",
        EngineCmd::CmdRenderGraphList(_) => "render-graph-list",
        EngineCmd::CmdRealmRenderGraphBind(_) => "realm-render-graph-bind",
        EngineCmd::CmdTargetUpsert(_) => "target-upsert",
        EngineCmd::CmdTargetMeasurement(_) => "target-measurement",
        EngineCmd::CmdTargetDispose(_) => "target-dispose",
        EngineCmd::CmdTargetLayerUpsert(_) => "target-layer-upsert",
        EngineCmd::CmdTargetLayerDispose(_) => "target-layer-dispose",
        EngineCmd::CmdUiThemeDefine(_) => "ui-theme-define",
        EngineCmd::CmdUiThemeDispose(_) => "ui-theme-dispose",
        EngineCmd::CmdUiDocumentCreate(_) => "ui-document-create",
        EngineCmd::CmdUiDocumentDispose(_) => "ui-document-dispose",
        EngineCmd::CmdUiDocumentSetRect(_) => "ui-document-set-rect",
        EngineCmd::CmdUiDocumentSetTheme(_) => "ui-document-set-theme",
        EngineCmd::CmdUiDocumentGetTree(_) => "ui-document-get-tree",
        EngineCmd::CmdUiDocumentGetLayoutRects(_) => "ui-document-get-layout-rects",
        EngineCmd::CmdUiApplyOps(_) => "ui-apply-ops",
        EngineCmd::CmdUiDebugSet(_) => "ui-debug-set",
        EngineCmd::CmdUiFocusSet(_) => "ui-focus-set",
        EngineCmd::CmdUiFocusGet(_) => "ui-focus-get",
        EngineCmd::CmdUiEventTraceSet(_) => "ui-event-trace-set",
        EngineCmd::CmdUiImageCreateFromBuffer(_) => "ui-image-create-from-buffer",
        EngineCmd::CmdUiImageDispose(_) => "ui-image-dispose",
        EngineCmd::CmdUiClipboardPaste(_) => "ui-clipboard-paste",
        EngineCmd::CmdUiScreenshotReply(_) => "ui-screenshot-reply",
        EngineCmd::CmdUiAccessKitActionRequest(_) => "ui-access-kit-action-request",
        EngineCmd::CmdModelList(_) => "model-list",
        EngineCmd::CmdMaterialList(_) => "material-list",
        EngineCmd::CmdTextureList(_) => "texture-list",
        EngineCmd::CmdGeometryList(_) => "geometry-list",
        EngineCmd::CmdLightList(_) => "light-list",
        EngineCmd::CmdCameraList(_) => "camera-list",
        EngineCmd::CmdGizmoDrawLine(_) => "gizmo-draw-line",
        EngineCmd::CmdGizmoDrawAabb(_) => "gizmo-draw-aabb",
        EngineCmd::CmdGizmoDrawPolyline(_) => "gizmo-draw-polyline",
    }
}

pub(super) fn emit_deferred_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
    reason: String,
) {
    engine
        .event_queue
        .push(EngineEvent::System(SystemEvent::CommandDeferred {
            command_id,
            command_type: command_type.to_string(),
            attempts,
            reason,
        }));
}

pub(super) fn emit_applied_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
) {
    engine
        .event_queue
        .push(EngineEvent::System(SystemEvent::CommandApplied {
            command_id,
            command_type: command_type.to_string(),
            attempts,
        }));
}

pub(super) fn emit_dropped_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
    reason: String,
) {
    engine
        .event_queue
        .push(EngineEvent::System(SystemEvent::CommandDropped {
            command_id,
            command_type: command_type.to_string(),
            attempts,
            reason,
        }));
}

fn command_has_pending_dependencies(engine: &EngineState, cmd: &EngineCmd) -> bool {
    match cmd {
        // Query/measurement commands must be non-blocking and should not be deferred.
        EngineCmd::CmdWindowMeasurement(_) => false,
        EngineCmd::CmdTargetMeasurement(_) => false,
        EngineCmd::CmdShadowConfigure(args) => {
            engine.device.is_none()
                || engine
                    .render
                    .get(&args.window_id)
                    .and_then(|state| state.shadow.as_ref())
                    .is_none()
        }
        EngineCmd::CmdTextureCreateSolidColor(_) => {
            engine.device.is_none() || engine.queue.is_none()
        }
        EngineCmd::CmdTextureCreateFromBuffer(args) => {
            !engine.buffers.uploads.contains_key(&args.buffer_id)
        }
        EngineCmd::CmdUiImageCreateFromBuffer(args) => {
            !engine.buffers.uploads.contains_key(&args.buffer_id)
        }
        EngineCmd::CmdPoseUpdate(args) => {
            let realm_id = crate::core::realm::RealmId(args.realm_id);
            let has_realm = engine
                .universal_state
                .realm_entities
                .contains_key(&realm_id);
            let has_model = engine
                .universal_state
                .realm_entities
                .get(&realm_id)
                .and_then(|entities| entities.models.get(&args.model_id))
                .is_some();
            let has_buffer = engine
                .buffers
                .uploads
                .contains_key(&args.matrices_buffer_id);
            let has_render = if let Some(window_id) = args.window_id {
                engine.render.states.contains_key(&window_id)
            } else {
                !engine.render.states.is_empty()
            };
            !has_realm || !has_model || !has_buffer || !has_render
        }
        EngineCmd::CmdCameraUpsert(args) => match args {
            CmdCameraUpsertArgs::Create(_) => false,
            CmdCameraUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .realm_entities
                    .get(&realm_id)
                    .and_then(|entities| entities.cameras.get(&update.camera_id))
                    .is_none()
            }
        },
        EngineCmd::CmdModelUpsert(args) => match args {
            CmdModelUpsertArgs::Create(_) => false,
            CmdModelUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .realm_entities
                    .get(&realm_id)
                    .and_then(|entities| entities.models.get(&update.model_id))
                    .is_none()
            }
        },
        EngineCmd::CmdLightUpsert(args) => match args {
            CmdLightUpsertArgs::Create(_) => false,
            CmdLightUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .realm_entities
                    .get(&realm_id)
                    .and_then(|entities| entities.lights.get(&update.light_id))
                    .is_none()
            }
        },
        EngineCmd::CmdMaterialUpsert(args) => match args {
            CmdMaterialUpsertArgs::Create(_) => false,
            CmdMaterialUpsertArgs::Update(update) => {
                let resources = &engine.universal_state.universal_resources;
                !(resources
                    .materials_standard
                    .contains_key(&update.material_id)
                    || resources.materials_pbr.contains_key(&update.material_id))
            }
        },
        EngineCmd::CmdGeometryUpsert(args) => match args {
            CmdGeometryUpsertArgs::Create(_) => false,
            CmdGeometryUpsertArgs::Update(update) => !engine
                .universal_state
                .universal_resources
                .geometries
                .contains_key(&update.geometry_id),
        },
        EngineCmd::CmdEnvironmentUpsert(args) => match args {
            CmdEnvironmentUpsertArgs::Create(_) => false,
            CmdEnvironmentUpsertArgs::Update(update) => !engine
                .universal_state
                .environment_profiles
                .contains_key(&update.environment_id),
        },
        EngineCmd::CmdTargetUpsert(args) => {
            matches!(args.kind, crate::core::target::TargetKind::Window)
                && args
                    .window_id
                    .is_some_and(|window_id| !engine.window.states.contains_key(&window_id))
        }
        EngineCmd::CmdTargetLayerUpsert(args) => {
            let realm_id = crate::core::realm::RealmId(args.realm_id);
            !engine
                .universal_state
                .realms
                .entries
                .contains_key(&realm_id)
                || !engine
                    .universal_state
                    .targets
                    .entries
                    .contains_key(&crate::core::target::TargetId(args.target_id))
        }
        EngineCmd::CmdUiDocumentSetRect(args) => !engine
            .universal_state
            .ui
            .documents
            .contains_key(&args.document_id),
        EngineCmd::CmdUiDocumentSetTheme(args) => !engine
            .universal_state
            .ui
            .documents
            .contains_key(&args.document_id),
        // Versioned UI ops are order-sensitive; replaying deferred stale ops can cause visual
        // oscillation. Keep them immediate (success/fail) instead of deferred.
        EngineCmd::CmdUiApplyOps(_) => false,
        EngineCmd::CmdUiFocusSet(args) => {
            let realm_id = crate::core::realm::RealmId(args.realm_id);
            !engine
                .universal_state
                .realms
                .entries
                .contains_key(&realm_id)
                || !engine
                    .universal_state
                    .ui
                    .documents
                    .contains_key(&args.document_id)
        }
        EngineCmd::CmdUiFocusGet(_) => false,
        EngineCmd::CmdUiClipboardPaste(args) => !engine
            .universal_state
            .ui
            .focus
            .realm_by_window
            .contains_key(&args.window_id),
        EngineCmd::CmdUiScreenshotReply(args) => {
            if let Some(realm_id) = args.realm_id {
                !engine
                    .universal_state
                    .ui
                    .realms
                    .contains_key(&crate::core::realm::RealmId(realm_id))
            } else {
                !engine
                    .universal_state
                    .ui
                    .focus
                    .realm_by_window
                    .contains_key(&args.window_id)
            }
        }
        EngineCmd::CmdUiAccessKitActionRequest(args) => {
            !engine.window.states.contains_key(&args.window_id)
        }
        _ => false,
    }
}

pub(super) fn classify_failed_response(
    engine: &EngineState,
    cmd: &EngineCmd,
    response: &CommandResponse,
) -> Option<(DeferredFailureKind, String)> {
    if super::response_maps::response_is_success(response) {
        return None;
    }

    let message = super::response_maps::response_message(response)
        .unwrap_or_else(|| "command failed".to_string());
    let kind = if command_has_pending_dependencies(engine, cmd) {
        DeferredFailureKind::Transient
    } else {
        DeferredFailureKind::Permanent
    };
    Some((kind, message))
}

pub(super) use vulfram_runtime::should_drop_deferred;
