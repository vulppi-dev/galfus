use super::super::*;
use crate::core::state::EngineState;

pub(super) fn maybe_emit_response_error_event(
    engine: &mut EngineState,
    command_id: u64,
    response: &CommandResponse,
) {
    macro_rules! failure_case {
        ($result:expr, $name:literal) => {{
            let result = $result;
            if !result.success {
                Some(($name, result.message.as_str()))
            } else {
                None
            }
        }};
    }

    let failure = match response {
        CommandResponse::UploadBufferDiscardAll(result) => {
            failure_case!(result, "upload-buffer-discard-all")
        }
        CommandResponse::SystemDiagnosticsSet(result) => {
            failure_case!(result, "system-diagnostics-set")
        }
        CommandResponse::CameraUpsert(result) => failure_case!(result, "camera-upsert"),
        CommandResponse::WindowCreate(result) => failure_case!(result, "window-create"),
        CommandResponse::WindowMeasurement(result) => failure_case!(result, "window-measurement"),
        CommandResponse::WindowCursor(result) => failure_case!(result, "window-cursor"),
        CommandResponse::WindowState(result) => failure_case!(result, "window-state"),
        CommandResponse::CameraDispose(result) => failure_case!(result, "camera-dispose"),
        CommandResponse::ModelUpsert(result) => failure_case!(result, "model-upsert"),
        CommandResponse::PoseUpdate(result) => failure_case!(result, "pose-update"),
        CommandResponse::ModelDispose(result) => failure_case!(result, "model-dispose"),
        CommandResponse::LightUpsert(result) => failure_case!(result, "light-upsert"),
        CommandResponse::LightDispose(result) => failure_case!(result, "light-dispose"),
        CommandResponse::MaterialUpsert(result) => failure_case!(result, "material-upsert"),
        CommandResponse::MaterialDispose(result) => failure_case!(result, "material-dispose"),
        CommandResponse::TextureCreateFromBuffer(result) => {
            failure_case!(result, "texture-create-from-buffer")
        }
        CommandResponse::TextureCreateSolidColor(result) => {
            failure_case!(result, "texture-create-solid-color")
        }
        CommandResponse::TextureDispose(result) => failure_case!(result, "texture-dispose"),
        CommandResponse::TextureBindTarget(result) => failure_case!(result, "texture-bind-target"),
        CommandResponse::AudioListenerUpsert(result) => {
            failure_case!(result, "audio-listener-upsert")
        }
        CommandResponse::AudioListenerDispose(result) => {
            failure_case!(result, "audio-listener-dispose")
        }
        CommandResponse::AudioResourceUpsert(result) => {
            failure_case!(result, "audio-resource-upsert")
        }
        CommandResponse::AudioSourceUpsert(result) => failure_case!(result, "audio-source-upsert"),
        CommandResponse::AudioSourceTransport(result) => {
            failure_case!(result, "audio-source-transport")
        }
        CommandResponse::AudioStateGet(result) => failure_case!(result, "audio-state-get"),
        CommandResponse::AudioSourceDispose(result) => {
            failure_case!(result, "audio-source-dispose")
        }
        CommandResponse::AudioResourceDispose(result) => {
            failure_case!(result, "audio-resource-dispose")
        }
        CommandResponse::GeometryUpsert(result) => failure_case!(result, "geometry-upsert"),
        CommandResponse::GeometryDispose(result) => failure_case!(result, "geometry-dispose"),
        CommandResponse::PrimitiveGeometryCreate(result) => {
            failure_case!(result, "primitive-geometry-create")
        }
        CommandResponse::EnvironmentUpsert(result) => failure_case!(result, "environment-upsert"),
        CommandResponse::EnvironmentDispose(result) => {
            failure_case!(result, "environment-dispose")
        }
        CommandResponse::ShadowConfigure(result) => failure_case!(result, "shadow-configure"),
        CommandResponse::RealmCreate(result) => failure_case!(result, "realm-create"),
        CommandResponse::RealmDispose(result) => failure_case!(result, "realm-dispose"),
        CommandResponse::TargetUpsert(result) => failure_case!(result, "target-upsert"),
        CommandResponse::TargetMeasurement(result) => failure_case!(result, "target-measurement"),
        CommandResponse::TargetDispose(result) => failure_case!(result, "target-dispose"),
        CommandResponse::TargetLayerUpsert(result) => {
            failure_case!(result, "target-layer-upsert")
        }
        CommandResponse::TargetLayerDispose(result) => {
            failure_case!(result, "target-layer-dispose")
        }
        CommandResponse::UiThemeDefine(result) => failure_case!(result, "ui-theme-define"),
        CommandResponse::UiThemeDispose(result) => failure_case!(result, "ui-theme-dispose"),
        CommandResponse::UiDocumentCreate(result) => failure_case!(result, "ui-document-create"),
        CommandResponse::UiDocumentDispose(result) => {
            failure_case!(result, "ui-document-dispose")
        }
        CommandResponse::UiDocumentSetRect(result) => {
            failure_case!(result, "ui-document-set-rect")
        }
        CommandResponse::UiDocumentSetTheme(result) => {
            failure_case!(result, "ui-document-set-theme")
        }
        CommandResponse::UiDocumentGetTree(result) => {
            failure_case!(result, "ui-document-get-tree")
        }
        CommandResponse::UiDocumentGetLayoutRects(result) => {
            failure_case!(result, "ui-document-get-layout-rects")
        }
        CommandResponse::UiApplyOps(result) => failure_case!(result, "ui-apply-ops"),
        CommandResponse::UiDebugSet(result) => failure_case!(result, "ui-debug-set"),
        CommandResponse::UiFocusSet(result) => failure_case!(result, "ui-focus-set"),
        CommandResponse::UiFocusGet(result) => failure_case!(result, "ui-focus-get"),
        CommandResponse::UiEventTraceSet(result) => {
            failure_case!(result, "ui-event-trace-set")
        }
        CommandResponse::UiImageCreateFromBuffer(result) => {
            failure_case!(result, "ui-image-create-from-buffer")
        }
        CommandResponse::UiImageDispose(result) => failure_case!(result, "ui-image-dispose"),
        CommandResponse::ModelList(result) => failure_case!(result, "model-list"),
        CommandResponse::MaterialList(result) => failure_case!(result, "material-list"),
        CommandResponse::TextureList(result) => failure_case!(result, "texture-list"),
        CommandResponse::GeometryList(result) => failure_case!(result, "geometry-list"),
        CommandResponse::LightList(result) => failure_case!(result, "light-list"),
        CommandResponse::CameraList(result) => failure_case!(result, "camera-list"),
        _ => None,
    };

    let Some((command_type, message)) = failure else {
        return;
    };

    sys::push_error_event(
        engine,
        "command",
        message.to_string(),
        Some(command_id),
        Some(command_type.to_string()),
    );
}
