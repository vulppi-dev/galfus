use super::super::*;

pub(super) fn response_is_success(response: &CommandResponse) -> bool {
    match response {
        CommandResponse::NotificationSend(result) => result.success,
        CommandResponse::SystemDiagnosticsSet(result) => result.success,
        CommandResponse::SystemBuildVersionGet(result) => result.success,
        CommandResponse::WindowCreate(result) => result.success,
        CommandResponse::WindowClose(_) => true,
        CommandResponse::WindowMeasurement(result) => result.success,
        CommandResponse::WindowCursor(result) => result.success,
        CommandResponse::WindowState(result) => result.success,
        CommandResponse::InputTargetListenerUpsert(result) => result.success,
        CommandResponse::InputTargetListenerDispose(result) => result.success,
        CommandResponse::InputTargetListenerList(result) => result.success,
        CommandResponse::UploadBufferDiscardAll(result) => result.success,
        CommandResponse::CameraUpsert(result) => result.success,
        CommandResponse::CameraDispose(result) => result.success,
        CommandResponse::ModelUpsert(result) => result.success,
        CommandResponse::PoseUpdate(result) => result.success,
        CommandResponse::ModelDispose(result) => result.success,
        CommandResponse::LightUpsert(result) => result.success,
        CommandResponse::LightDispose(result) => result.success,
        CommandResponse::MaterialUpsert(result) => result.success,
        CommandResponse::MaterialDispose(result) => result.success,
        CommandResponse::TextureCreateFromBuffer(result) => result.success,
        CommandResponse::TextureCreateSolidColor(result) => result.success,
        CommandResponse::TextureDispose(result) => result.success,
        CommandResponse::TextureBindTarget(result) => result.success,
        CommandResponse::AudioListenerUpsert(result) => result.success,
        CommandResponse::AudioListenerDispose(result) => result.success,
        CommandResponse::AudioResourceUpsert(result) => result.success,
        CommandResponse::AudioSourceUpsert(result) => result.success,
        CommandResponse::AudioSourceTransport(result) => result.success,
        CommandResponse::AudioStateGet(result) => result.success,
        CommandResponse::AudioSourceDispose(result) => result.success,
        CommandResponse::AudioResourceDispose(result) => result.success,
        CommandResponse::GeometryUpsert(result) => result.success,
        CommandResponse::GeometryDispose(result) => result.success,
        CommandResponse::PrimitiveGeometryCreate(result) => result.success,
        CommandResponse::EnvironmentUpsert(result) => result.success,
        CommandResponse::EnvironmentDispose(result) => result.success,
        CommandResponse::ShadowConfigure(result) => result.success,
        CommandResponse::RealmCreate(result) => result.success,
        CommandResponse::RealmDispose(result) => result.success,
        CommandResponse::TargetUpsert(result) => result.success,
        CommandResponse::TargetDispose(result) => result.success,
        CommandResponse::TargetLayerUpsert(result) => result.success,
        CommandResponse::TargetLayerDispose(result) => result.success,
        CommandResponse::UiThemeDefine(result) => result.success,
        CommandResponse::UiThemeDispose(result) => result.success,
        CommandResponse::UiDocumentCreate(result) => result.success,
        CommandResponse::UiDocumentDispose(result) => result.success,
        CommandResponse::UiDocumentSetRect(result) => result.success,
        CommandResponse::UiDocumentSetTheme(result) => result.success,
        CommandResponse::UiDocumentGetTree(result) => result.success,
        CommandResponse::UiDocumentGetLayoutRects(result) => result.success,
        CommandResponse::UiApplyOps(result) => result.success,
        CommandResponse::UiDebugSet(result) => result.success,
        CommandResponse::UiFocusSet(result) => result.success,
        CommandResponse::UiFocusGet(result) => result.success,
        CommandResponse::UiEventTraceSet(result) => result.success,
        CommandResponse::UiImageCreateFromBuffer(result) => result.success,
        CommandResponse::UiImageDispose(result) => result.success,
        CommandResponse::UiClipboardPaste(result) => result.success,
        CommandResponse::UiScreenshotReply(result) => result.success,
        CommandResponse::UiAccessKitActionRequest(result) => result.success,
        CommandResponse::ModelList(result) => result.success,
        CommandResponse::MaterialList(result) => result.success,
        CommandResponse::TextureList(result) => result.success,
        CommandResponse::GeometryList(result) => result.success,
        CommandResponse::LightList(result) => result.success,
        CommandResponse::CameraList(result) => result.success,
        CommandResponse::GizmoDrawLine(result) => result.status == 0,
        CommandResponse::GizmoDrawAabb(result) => result.status == 0,
    }
}

pub(super) fn response_message(response: &CommandResponse) -> Option<String> {
    match response {
        CommandResponse::NotificationSend(_) => None,
        CommandResponse::SystemDiagnosticsSet(result) => Some(result.message.clone()),
        CommandResponse::SystemBuildVersionGet(result) => Some(result.message.clone()),
        CommandResponse::WindowCreate(result) => Some(result.message.clone()),
        CommandResponse::WindowClose(_) => None,
        CommandResponse::WindowMeasurement(result) => Some(result.message.clone()),
        CommandResponse::WindowCursor(result) => Some(result.message.clone()),
        CommandResponse::WindowState(result) => Some(result.message.clone()),
        CommandResponse::InputTargetListenerUpsert(result) => Some(result.message.clone()),
        CommandResponse::InputTargetListenerDispose(result) => Some(result.message.clone()),
        CommandResponse::InputTargetListenerList(result) => Some(result.message.clone()),
        CommandResponse::UploadBufferDiscardAll(result) => Some(result.message.clone()),
        CommandResponse::CameraUpsert(result) => Some(result.message.clone()),
        CommandResponse::CameraDispose(result) => Some(result.message.clone()),
        CommandResponse::ModelUpsert(result) => Some(result.message.clone()),
        CommandResponse::PoseUpdate(result) => Some(result.message.clone()),
        CommandResponse::ModelDispose(result) => Some(result.message.clone()),
        CommandResponse::LightUpsert(result) => Some(result.message.clone()),
        CommandResponse::LightDispose(result) => Some(result.message.clone()),
        CommandResponse::MaterialUpsert(result) => Some(result.message.clone()),
        CommandResponse::MaterialDispose(result) => Some(result.message.clone()),
        CommandResponse::TextureCreateFromBuffer(result) => Some(result.message.clone()),
        CommandResponse::TextureCreateSolidColor(result) => Some(result.message.clone()),
        CommandResponse::TextureDispose(result) => Some(result.message.clone()),
        CommandResponse::TextureBindTarget(result) => Some(result.message.clone()),
        CommandResponse::AudioListenerUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioListenerDispose(result) => Some(result.message.clone()),
        CommandResponse::AudioResourceUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceTransport(result) => Some(result.message.clone()),
        CommandResponse::AudioStateGet(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceDispose(result) => Some(result.message.clone()),
        CommandResponse::AudioResourceDispose(result) => Some(result.message.clone()),
        CommandResponse::GeometryUpsert(result) => Some(result.message.clone()),
        CommandResponse::GeometryDispose(result) => Some(result.message.clone()),
        CommandResponse::PrimitiveGeometryCreate(result) => Some(result.message.clone()),
        CommandResponse::EnvironmentUpsert(result) => Some(result.message.clone()),
        CommandResponse::EnvironmentDispose(result) => Some(result.message.clone()),
        CommandResponse::ShadowConfigure(result) => Some(result.message.clone()),
        CommandResponse::RealmCreate(result) => Some(result.message.clone()),
        CommandResponse::RealmDispose(result) => Some(result.message.clone()),
        CommandResponse::TargetUpsert(result) => Some(result.message.clone()),
        CommandResponse::TargetDispose(result) => Some(result.message.clone()),
        CommandResponse::TargetLayerUpsert(result) => Some(result.message.clone()),
        CommandResponse::TargetLayerDispose(result) => Some(result.message.clone()),
        CommandResponse::UiThemeDefine(result) => Some(result.message.clone()),
        CommandResponse::UiThemeDispose(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentCreate(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentDispose(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentSetRect(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentSetTheme(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentGetTree(result) => Some(result.message.clone()),
        CommandResponse::UiDocumentGetLayoutRects(result) => Some(result.message.clone()),
        CommandResponse::UiApplyOps(result) => Some(result.message.clone()),
        CommandResponse::UiDebugSet(result) => Some(result.message.clone()),
        CommandResponse::UiFocusSet(result) => Some(result.message.clone()),
        CommandResponse::UiFocusGet(result) => Some(result.message.clone()),
        CommandResponse::UiEventTraceSet(result) => Some(result.message.clone()),
        CommandResponse::UiImageCreateFromBuffer(result) => Some(result.message.clone()),
        CommandResponse::UiImageDispose(result) => Some(result.message.clone()),
        CommandResponse::UiClipboardPaste(result) => Some(result.message.clone()),
        CommandResponse::UiScreenshotReply(result) => Some(result.message.clone()),
        CommandResponse::UiAccessKitActionRequest(result) => Some(result.message.clone()),
        CommandResponse::ModelList(result) => Some(result.message.clone()),
        CommandResponse::MaterialList(result) => Some(result.message.clone()),
        CommandResponse::TextureList(result) => Some(result.message.clone()),
        CommandResponse::GeometryList(result) => Some(result.message.clone()),
        CommandResponse::LightList(result) => Some(result.message.clone()),
        CommandResponse::CameraList(result) => Some(result.message.clone()),
        CommandResponse::GizmoDrawLine(_) => None,
        CommandResponse::GizmoDrawAabb(_) => None,
    }
}

pub(super) fn response_with_message(response: CommandResponse, message: String) -> CommandResponse {
    match response {
        CommandResponse::SystemDiagnosticsSet(mut result) => {
            result.message = message;
            CommandResponse::SystemDiagnosticsSet(result)
        }
        CommandResponse::SystemBuildVersionGet(mut result) => {
            result.message = message;
            CommandResponse::SystemBuildVersionGet(result)
        }
        CommandResponse::WindowCreate(mut result) => {
            result.message = message;
            CommandResponse::WindowCreate(result)
        }
        CommandResponse::WindowMeasurement(mut result) => {
            result.message = message;
            CommandResponse::WindowMeasurement(result)
        }
        CommandResponse::WindowCursor(mut result) => {
            result.message = message;
            CommandResponse::WindowCursor(result)
        }
        CommandResponse::WindowState(mut result) => {
            result.message = message;
            CommandResponse::WindowState(result)
        }
        CommandResponse::InputTargetListenerUpsert(mut result) => {
            result.message = message;
            CommandResponse::InputTargetListenerUpsert(result)
        }
        CommandResponse::InputTargetListenerDispose(mut result) => {
            result.message = message;
            CommandResponse::InputTargetListenerDispose(result)
        }
        CommandResponse::InputTargetListenerList(mut result) => {
            result.message = message;
            CommandResponse::InputTargetListenerList(result)
        }
        CommandResponse::UploadBufferDiscardAll(mut result) => {
            result.message = message;
            CommandResponse::UploadBufferDiscardAll(result)
        }
        CommandResponse::CameraUpsert(mut result) => {
            result.message = message;
            CommandResponse::CameraUpsert(result)
        }
        CommandResponse::CameraDispose(mut result) => {
            result.message = message;
            CommandResponse::CameraDispose(result)
        }
        CommandResponse::ModelUpsert(mut result) => {
            result.message = message;
            CommandResponse::ModelUpsert(result)
        }
        CommandResponse::PoseUpdate(mut result) => {
            result.message = message;
            CommandResponse::PoseUpdate(result)
        }
        CommandResponse::ModelDispose(mut result) => {
            result.message = message;
            CommandResponse::ModelDispose(result)
        }
        CommandResponse::LightUpsert(mut result) => {
            result.message = message;
            CommandResponse::LightUpsert(result)
        }
        CommandResponse::LightDispose(mut result) => {
            result.message = message;
            CommandResponse::LightDispose(result)
        }
        CommandResponse::MaterialUpsert(mut result) => {
            result.message = message;
            CommandResponse::MaterialUpsert(result)
        }
        CommandResponse::MaterialDispose(mut result) => {
            result.message = message;
            CommandResponse::MaterialDispose(result)
        }
        CommandResponse::TextureCreateFromBuffer(mut result) => {
            result.message = message;
            CommandResponse::TextureCreateFromBuffer(result)
        }
        CommandResponse::TextureCreateSolidColor(mut result) => {
            result.message = message;
            CommandResponse::TextureCreateSolidColor(result)
        }
        CommandResponse::TextureDispose(mut result) => {
            result.message = message;
            CommandResponse::TextureDispose(result)
        }
        CommandResponse::TextureBindTarget(mut result) => {
            result.message = message;
            CommandResponse::TextureBindTarget(result)
        }
        CommandResponse::AudioListenerUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioListenerUpsert(result)
        }
        CommandResponse::AudioListenerDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioListenerDispose(result)
        }
        CommandResponse::AudioResourceUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioResourceUpsert(result)
        }
        CommandResponse::AudioSourceUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceUpsert(result)
        }
        CommandResponse::AudioSourceTransport(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceTransport(result)
        }
        CommandResponse::AudioStateGet(mut result) => {
            result.message = message;
            CommandResponse::AudioStateGet(result)
        }
        CommandResponse::AudioSourceDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceDispose(result)
        }
        CommandResponse::AudioResourceDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioResourceDispose(result)
        }
        CommandResponse::GeometryUpsert(mut result) => {
            result.message = message;
            CommandResponse::GeometryUpsert(result)
        }
        CommandResponse::GeometryDispose(mut result) => {
            result.message = message;
            CommandResponse::GeometryDispose(result)
        }
        CommandResponse::PrimitiveGeometryCreate(mut result) => {
            result.message = message;
            CommandResponse::PrimitiveGeometryCreate(result)
        }
        CommandResponse::EnvironmentUpsert(mut result) => {
            result.message = message;
            CommandResponse::EnvironmentUpsert(result)
        }
        CommandResponse::EnvironmentDispose(mut result) => {
            result.message = message;
            CommandResponse::EnvironmentDispose(result)
        }
        CommandResponse::ShadowConfigure(mut result) => {
            result.message = message;
            CommandResponse::ShadowConfigure(result)
        }
        CommandResponse::RealmCreate(mut result) => {
            result.message = message;
            CommandResponse::RealmCreate(result)
        }
        CommandResponse::RealmDispose(mut result) => {
            result.message = message;
            CommandResponse::RealmDispose(result)
        }
        CommandResponse::TargetUpsert(mut result) => {
            result.message = message;
            CommandResponse::TargetUpsert(result)
        }
        CommandResponse::TargetDispose(mut result) => {
            result.message = message;
            CommandResponse::TargetDispose(result)
        }
        CommandResponse::TargetLayerUpsert(mut result) => {
            result.message = message;
            CommandResponse::TargetLayerUpsert(result)
        }
        CommandResponse::TargetLayerDispose(mut result) => {
            result.message = message;
            CommandResponse::TargetLayerDispose(result)
        }
        CommandResponse::UiThemeDefine(mut result) => {
            result.message = message;
            CommandResponse::UiThemeDefine(result)
        }
        CommandResponse::UiThemeDispose(mut result) => {
            result.message = message;
            CommandResponse::UiThemeDispose(result)
        }
        CommandResponse::UiDocumentCreate(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentCreate(result)
        }
        CommandResponse::UiDocumentDispose(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentDispose(result)
        }
        CommandResponse::UiDocumentSetRect(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentSetRect(result)
        }
        CommandResponse::UiDocumentSetTheme(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentSetTheme(result)
        }
        CommandResponse::UiDocumentGetTree(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentGetTree(result)
        }
        CommandResponse::UiDocumentGetLayoutRects(mut result) => {
            result.message = message;
            CommandResponse::UiDocumentGetLayoutRects(result)
        }
        CommandResponse::UiApplyOps(mut result) => {
            result.message = message;
            CommandResponse::UiApplyOps(result)
        }
        CommandResponse::UiDebugSet(mut result) => {
            result.message = message;
            CommandResponse::UiDebugSet(result)
        }
        CommandResponse::UiFocusSet(mut result) => {
            result.message = message;
            CommandResponse::UiFocusSet(result)
        }
        CommandResponse::UiFocusGet(mut result) => {
            result.message = message;
            CommandResponse::UiFocusGet(result)
        }
        CommandResponse::UiEventTraceSet(mut result) => {
            result.message = message;
            CommandResponse::UiEventTraceSet(result)
        }
        CommandResponse::UiImageCreateFromBuffer(mut result) => {
            result.message = message;
            CommandResponse::UiImageCreateFromBuffer(result)
        }
        CommandResponse::UiImageDispose(mut result) => {
            result.message = message;
            CommandResponse::UiImageDispose(result)
        }
        CommandResponse::UiClipboardPaste(mut result) => {
            result.message = message;
            CommandResponse::UiClipboardPaste(result)
        }
        CommandResponse::UiScreenshotReply(mut result) => {
            result.message = message;
            CommandResponse::UiScreenshotReply(result)
        }
        CommandResponse::UiAccessKitActionRequest(mut result) => {
            result.message = message;
            CommandResponse::UiAccessKitActionRequest(result)
        }
        CommandResponse::ModelList(mut result) => {
            result.message = message;
            CommandResponse::ModelList(result)
        }
        CommandResponse::MaterialList(mut result) => {
            result.message = message;
            CommandResponse::MaterialList(result)
        }
        CommandResponse::TextureList(mut result) => {
            result.message = message;
            CommandResponse::TextureList(result)
        }
        CommandResponse::GeometryList(mut result) => {
            result.message = message;
            CommandResponse::GeometryList(result)
        }
        CommandResponse::LightList(mut result) => {
            result.message = message;
            CommandResponse::LightList(result)
        }
        CommandResponse::CameraList(mut result) => {
            result.message = message;
            CommandResponse::CameraList(result)
        }
        other => other,
    }
}
