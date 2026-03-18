use serde::{Deserialize, Serialize};

use crate::core::gamepad::events::GamepadEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::input::listeners::{
    CmdInputTargetListenerDisposeArgs, CmdInputTargetListenerListArgs,
    CmdInputTargetListenerUpsertArgs, CmdResultInputTargetListenerList,
};
use crate::core::system::SystemEvent;
use crate::core::ui::events::UiEvent;
use crate::core::window::WindowEvent;

pub use crate::core::audio;
pub use crate::core::buffers as buf;
pub use crate::core::realm::cmd as realm;
pub use crate::core::render::gizmos as gizmo;
pub use crate::core::resources as res;
pub use crate::core::system as sys;
pub use crate::core::target::cmd as target;
pub use crate::core::ui::cmd as ui;
pub use crate::core::window as win;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdCameraUpsertArgs {
    Create(res::CmdCameraCreateArgs),
    Update(res::CmdCameraUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdModelUpsertArgs {
    Create(res::CmdModelCreateArgs),
    Update(res::CmdModelUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdLightUpsertArgs {
    Create(res::CmdLightCreateArgs),
    Update(res::CmdLightUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdMaterialUpsertArgs {
    Create(res::CmdMaterialCreateArgs),
    Update(res::CmdMaterialUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdGeometryUpsertArgs {
    Create(res::CmdGeometryCreateArgs),
    Update(res::CmdGeometryUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdEnvironmentUpsertArgs {
    Create(res::CmdEnvironmentCreateArgs),
    Update(res::CmdEnvironmentUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdAudioListenerUpsertArgs {
    Create(audio::CmdAudioListenerCreateArgs),
    Update(audio::CmdAudioListenerUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdAudioSourceUpsertArgs {
    Create(audio::CmdAudioSourceCreateArgs),
    Update(audio::CmdAudioSourceUpdateArgs),
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSimple {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdNotificationSend(sys::CmdNotificationSendArgs),
    CmdSystemDiagnosticsSet(sys::CmdSystemDiagnosticsSetArgs),
    CmdSystemBuildVersionGet(sys::CmdSystemBuildVersionGetArgs),
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowMeasurement(win::CmdWindowMeasurementArgs),
    CmdWindowCursor(win::CmdWindowCursorArgs),
    CmdWindowState(win::CmdWindowStateArgs),
    CmdInputTargetListenerUpsert(CmdInputTargetListenerUpsertArgs),
    CmdInputTargetListenerDispose(CmdInputTargetListenerDisposeArgs),
    CmdInputTargetListenerList(CmdInputTargetListenerListArgs),
    CmdUploadBufferDiscardAll(buf::CmdUploadBufferDiscardAllArgs),
    CmdCameraUpsert(CmdCameraUpsertArgs),
    CmdCameraDispose(res::CmdCameraDisposeArgs),
    CmdModelUpsert(CmdModelUpsertArgs),
    CmdPoseUpdate(res::CmdPoseUpdateArgs),
    CmdModelDispose(res::CmdModelDisposeArgs),
    CmdLightUpsert(CmdLightUpsertArgs),
    CmdLightDispose(res::CmdLightDisposeArgs),
    CmdMaterialUpsert(CmdMaterialUpsertArgs),
    CmdMaterialDispose(res::CmdMaterialDisposeArgs),
    CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs),
    CmdTextureCreateSolidColor(res::CmdTextureCreateSolidColorArgs),
    CmdTextureDispose(res::CmdTextureDisposeArgs),
    CmdTextureBindTarget(res::CmdTextureBindTargetArgs),
    CmdAudioListenerUpsert(CmdAudioListenerUpsertArgs),
    CmdAudioListenerDispose(audio::CmdAudioListenerDisposeArgs),
    CmdAudioResourceUpsert(audio::CmdAudioResourceUpsertArgs),
    CmdAudioSourceUpsert(CmdAudioSourceUpsertArgs),
    CmdAudioSourceTransport(audio::CmdAudioSourceTransportArgs),
    CmdAudioStateGet(audio::CmdAudioStateGetArgs),
    CmdAudioSourceDispose(audio::CmdAudioSourceDisposeArgs),
    CmdAudioResourceDispose(audio::CmdAudioResourceDisposeArgs),
    CmdGeometryUpsert(CmdGeometryUpsertArgs),
    CmdGeometryDispose(res::CmdGeometryDisposeArgs),
    CmdPrimitiveGeometryCreate(res::CmdPrimitiveGeometryCreateArgs),
    CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs),
    CmdEnvironmentDispose(res::CmdEnvironmentDisposeArgs),
    CmdShadowConfigure(res::shadow::CmdShadowConfigureArgs),
    CmdRealmCreate(realm::CmdRealmCreateArgs),
    CmdRealmDispose(realm::CmdRealmDisposeArgs),
    CmdRenderGraphUpsert(realm::CmdRenderGraphUpsertArgs),
    CmdRenderGraphDispose(realm::CmdRenderGraphDisposeArgs),
    CmdRenderGraphList(realm::CmdRenderGraphListArgs),
    CmdRealmRenderGraphBind(realm::CmdRealmRenderGraphBindArgs),
    CmdTargetUpsert(target::CmdTargetUpsertArgs),
    CmdTargetMeasurement(target::CmdTargetMeasurementArgs),
    CmdTargetDispose(target::CmdTargetDisposeArgs),
    CmdTargetLayerUpsert(target::CmdTargetLayerUpsertArgs),
    CmdTargetLayerDispose(target::CmdTargetLayerDisposeArgs),
    CmdUiThemeDefine(ui::CmdUiThemeDefineArgs),
    CmdUiThemeDispose(ui::CmdUiThemeDisposeArgs),
    CmdUiDocumentCreate(ui::CmdUiDocumentCreateArgs),
    CmdUiDocumentDispose(ui::CmdUiDocumentDisposeArgs),
    CmdUiDocumentSetRect(ui::CmdUiDocumentSetRectArgs),
    CmdUiDocumentSetTheme(ui::CmdUiDocumentSetThemeArgs),
    CmdUiDocumentGetTree(ui::CmdUiDocumentGetTreeArgs),
    CmdUiDocumentGetLayoutRects(ui::CmdUiDocumentGetLayoutRectsArgs),
    CmdUiApplyOps(ui::CmdUiApplyOpsArgs),
    CmdUiDebugSet(ui::CmdUiDebugSetArgs),
    CmdUiFocusSet(ui::CmdUiFocusSetArgs),
    CmdUiFocusGet(ui::CmdUiFocusGetArgs),
    CmdUiEventTraceSet(ui::CmdUiEventTraceSetArgs),
    CmdUiImageCreateFromBuffer(ui::CmdUiImageCreateFromBufferArgs),
    CmdUiImageDispose(ui::CmdUiImageDisposeArgs),
    CmdUiClipboardPaste(ui::CmdUiClipboardPasteArgs),
    CmdUiScreenshotReply(ui::CmdUiScreenshotReplyArgs),
    CmdUiAccessKitActionRequest(ui::CmdUiAccessKitActionRequestArgs),
    CmdModelList(res::CmdModelListArgs),
    CmdMaterialList(res::CmdMaterialListArgs),
    CmdTextureList(res::CmdTextureListArgs),
    CmdGeometryList(res::CmdGeometryListArgs),
    CmdLightList(res::CmdLightListArgs),
    CmdCameraList(res::CmdCameraListArgs),
    CmdGizmoDrawLine(gizmo::CmdGizmoDrawLineArgs),
    CmdGizmoDrawAabb(gizmo::CmdGizmoDrawAabbArgs),
}

/// Spontaneous engine events (input, window changes, system events)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(WindowEvent),
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    Gamepad(GamepadEvent),
    System(SystemEvent),
    Ui(UiEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
    NotificationSend(sys::CmdResultNotificationSend),
    SystemDiagnosticsSet(sys::CmdResultSystemDiagnosticsSet),
    SystemBuildVersionGet(sys::CmdResultSystemBuildVersionGet),
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowMeasurement(win::CmdResultWindowMeasurement),
    WindowCursor(win::CmdResultWindowCursor),
    WindowState(win::CmdResultWindowState),
    InputTargetListenerUpsert(CmdResultSimple),
    InputTargetListenerDispose(CmdResultSimple),
    InputTargetListenerList(CmdResultInputTargetListenerList),
    UploadBufferDiscardAll(buf::CmdResultUploadBufferDiscardAll),
    CameraUpsert(CmdResultSimple),
    CameraDispose(res::CmdResultCameraDispose),
    ModelUpsert(CmdResultSimple),
    PoseUpdate(res::CmdResultPoseUpdate),
    ModelDispose(res::CmdResultModelDispose),
    LightUpsert(CmdResultSimple),
    LightDispose(res::CmdResultLightDispose),
    MaterialUpsert(CmdResultSimple),
    MaterialDispose(res::CmdResultMaterialDispose),
    TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer),
    TextureCreateSolidColor(res::CmdResultTextureCreateSolidColor),
    TextureDispose(res::CmdResultTextureDispose),
    TextureBindTarget(res::CmdResultTextureBindTarget),
    AudioListenerUpsert(CmdResultSimple),
    AudioListenerDispose(audio::CmdResultAudioListenerDispose),
    AudioResourceUpsert(audio::CmdResultAudioResourceUpsert),
    AudioSourceUpsert(CmdResultSimple),
    AudioSourceTransport(audio::CmdResultAudioSourceTransport),
    AudioStateGet(audio::CmdResultAudioStateGet),
    AudioSourceDispose(audio::CmdResultAudioSourceDispose),
    AudioResourceDispose(audio::CmdResultAudioResourceDispose),
    GeometryUpsert(CmdResultSimple),
    GeometryDispose(res::CmdResultGeometryDispose),
    PrimitiveGeometryCreate(res::CmdResultPrimitiveGeometryCreate),
    EnvironmentUpsert(CmdResultSimple),
    EnvironmentDispose(res::CmdResultEnvironment),
    ShadowConfigure(res::shadow::CmdResultShadowConfigure),
    RealmCreate(realm::CmdResultRealmCreate),
    RealmDispose(realm::CmdResultRealmDispose),
    RenderGraphUpsert(realm::CmdResultRenderGraphUpsert),
    RenderGraphDispose(realm::CmdResultRenderGraphDispose),
    RenderGraphList(realm::CmdResultRenderGraphList),
    RealmRenderGraphBind(realm::CmdResultRealmRenderGraphBind),
    TargetUpsert(target::CmdResultTargetUpsert),
    TargetMeasurement(target::CmdResultTargetMeasurement),
    TargetDispose(target::CmdResultTargetDispose),
    TargetLayerUpsert(target::CmdResultTargetLayerUpsert),
    TargetLayerDispose(target::CmdResultTargetLayerDispose),
    UiThemeDefine(ui::CmdResultUiThemeDefine),
    UiThemeDispose(ui::CmdResultUiThemeDispose),
    UiDocumentCreate(ui::CmdResultUiDocumentCreate),
    UiDocumentDispose(ui::CmdResultUiDocumentDispose),
    UiDocumentSetRect(ui::CmdResultUiDocumentSetRect),
    UiDocumentSetTheme(ui::CmdResultUiDocumentSetTheme),
    UiDocumentGetTree(ui::CmdResultUiDocumentGetTree),
    UiDocumentGetLayoutRects(ui::CmdResultUiDocumentGetLayoutRects),
    UiApplyOps(ui::CmdResultUiApplyOps),
    UiDebugSet(ui::CmdResultUiDebugSet),
    UiFocusSet(ui::CmdResultUiFocusSet),
    UiFocusGet(ui::CmdResultUiFocusGet),
    UiEventTraceSet(ui::CmdResultUiEventTraceSet),
    UiImageCreateFromBuffer(ui::CmdResultUiImageCreateFromBuffer),
    UiImageDispose(ui::CmdResultUiImageDispose),
    UiClipboardPaste(ui::CmdResultUiInputEvent),
    UiScreenshotReply(ui::CmdResultUiInputEvent),
    UiAccessKitActionRequest(ui::CmdResultUiInputEvent),
    ModelList(res::CmdResultModelList),
    MaterialList(res::CmdResultMaterialList),
    TextureList(res::CmdResultTextureList),
    GeometryList(res::CmdResultGeometryList),
    LightList(res::CmdResultLightList),
    CameraList(res::CmdResultCameraList),
    GizmoDrawLine(gizmo::CmdResultGizmoDraw),
    GizmoDrawAabb(gizmo::CmdResultGizmoDraw),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponseEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub response: CommandResponse,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEvent>;

pub type EngineBatchResponses = Vec<CommandResponseEnvelope>;

mod processing;

pub(crate) use processing::{deferred_command_key, engine_process_batch};
