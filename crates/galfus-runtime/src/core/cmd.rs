use serde::{Deserialize, Serialize};

use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::system::SystemEvent;
use crate::core::window::WindowEvent;
use galfus_input::GamepadEvent;

pub use crate::core::audio;
pub use crate::core::buffers as buf;
pub use crate::core::realm::cmd as realm;
pub use crate::core::render::gizmos as gizmo;
pub use crate::core::resources as res;
pub use crate::core::system as sys;
pub use crate::core::target::cmd as target;
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
pub enum CmdMaterialDefinitionUpsertArgs {
    Create(res::CmdMaterialDefinitionCreateArgs),
    Update(res::CmdMaterialDefinitionUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdMaterialInstanceUpsertArgs {
    Create(res::CmdMaterialInstanceCreateArgs),
    Update(res::CmdMaterialInstanceUpdateArgs),
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdTextureUpsertArgs {
    FromBuffer(res::CmdTextureCreateFromBufferArgs),
    SolidColor(res::CmdTextureCreateSolidColorArgs),
}

pub type CmdResultSimple = galfus_protocol::CmdResultSimple;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdNotificationSend(sys::CmdNotificationSendArgs),
    CmdSystemDiagnosticsSet(sys::CmdSystemDiagnosticsSetArgs),
    CmdSystemLogLevelSet(sys::CmdSystemLogLevelSetArgs),
    CmdSystemLogLevelGet(sys::CmdSystemLogLevelGetArgs),
    CmdSystemBuildVersionGet(sys::CmdSystemBuildVersionGetArgs),
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowMeasurement(win::CmdWindowMeasurementArgs),
    CmdWindowCursor(win::CmdWindowCursorArgs),
    CmdWindowState(win::CmdWindowStateArgs),
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
    CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs),
    CmdMaterialDefinitionDispose(res::CmdMaterialDefinitionDisposeArgs),
    CmdMaterialInstanceUpsert(CmdMaterialInstanceUpsertArgs),
    CmdMaterialInstanceDispose(res::CmdMaterialInstanceDisposeArgs),
    CmdMaterialDefinitionGet(res::CmdResourceGetArgs),
    CmdMaterialDefinitionList(res::CmdResourceListArgs),
    CmdMaterialInstanceGet(res::CmdResourceGetArgs),
    CmdMaterialInstanceList(res::CmdResourceListArgs),
    CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs),
    CmdTextureCreateSolidColor(res::CmdTextureCreateSolidColorArgs),
    CmdTextureUpsert(CmdTextureUpsertArgs),
    CmdTextureDispose(res::CmdTextureDisposeArgs),
    CmdTextureBindTarget(res::CmdTextureBindTargetArgs),
    CmdTextureGet(res::CmdResourceGetArgs),
    CmdAudioListenerUpsert(CmdAudioListenerUpsertArgs),
    CmdAudioListenerDispose(audio::CmdAudioListenerDisposeArgs),
    CmdAudioListenerGet(audio::CmdAudioListenerGetArgs),
    CmdAudioResourceUpsert(audio::CmdAudioResourceUpsertArgs),
    CmdAudioResourceGet(audio::CmdAudioResourceGetArgs),
    CmdAudioResourceList(audio::CmdAudioResourceListArgs),
    CmdAudioSourceUpsert(CmdAudioSourceUpsertArgs),
    CmdAudioSourceGet(audio::CmdAudioSourceGetArgs),
    CmdAudioSourceList(audio::CmdAudioSourceListArgs),
    CmdAudioSourceTransport(audio::CmdAudioSourceTransportArgs),
    CmdAudioStateGet(audio::CmdAudioStateGetArgs),
    CmdAudioSourceDispose(audio::CmdAudioSourceDisposeArgs),
    CmdAudioResourceDispose(audio::CmdAudioResourceDisposeArgs),
    CmdGeometryUpsert(CmdGeometryUpsertArgs),
    CmdGeometryDispose(res::CmdGeometryDisposeArgs),
    CmdGeometryGet(res::CmdResourceGetArgs),
    CmdPrimitiveGeometryCreate(res::CmdPrimitiveGeometryCreateArgs),
    CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs),
    CmdEnvironmentDispose(res::CmdEnvironmentDisposeArgs),
    CmdEnvironmentGet(res::CmdResourceGetArgs),
    CmdEnvironmentList(res::CmdResourceListArgs),
    CmdShadowConfigure(res::shadow::CmdShadowConfigureArgs),
    CmdRealmCreate(realm::CmdRealmCreateArgs),
    CmdRealmDispose(realm::CmdRealmDisposeArgs),
    CmdRealmGet(realm::CmdRealmGetArgs),
    CmdRealmList(realm::CmdRealmListArgs),
    CmdRenderGraphUpsert(realm::CmdRenderGraphUpsertArgs),
    CmdRenderGraphDispose(realm::CmdRenderGraphDisposeArgs),
    CmdRenderGraphList(realm::CmdRenderGraphListArgs),
    CmdRealmRenderGraphBind(realm::CmdRealmRenderGraphBindArgs),
    CmdTargetUpsert(target::CmdTargetUpsertArgs),
    CmdTargetGet(target::CmdTargetGetArgs),
    CmdTargetList(target::CmdTargetListArgs),
    CmdTargetMeasurement(target::CmdTargetMeasurementArgs),
    CmdTargetDispose(target::CmdTargetDisposeArgs),
    CmdTargetLayerUpsert(target::CmdTargetLayerUpsertArgs),
    CmdTargetLayerDispose(target::CmdTargetLayerDisposeArgs),
    CmdTargetLayerGet(target::CmdTargetLayerGetArgs),
    CmdTargetLayerList(target::CmdTargetLayerListArgs),
    CmdModelGet(res::CmdResourceGetArgs),
    CmdModelList(res::CmdModelListArgs),
    CmdMaterialGet(res::CmdResourceGetArgs),
    CmdMaterialList(res::CmdMaterialListArgs),
    CmdTextureList(res::CmdTextureListArgs),
    CmdGeometryList(res::CmdGeometryListArgs),
    CmdLightGet(res::CmdResourceGetArgs),
    CmdLightList(res::CmdLightListArgs),
    CmdCameraGet(res::CmdResourceGetArgs),
    CmdCameraList(res::CmdCameraListArgs),
    CmdGizmoDrawLine(gizmo::CmdGizmoDrawLineArgs),
    CmdGizmoDrawAabb(gizmo::CmdGizmoDrawAabbArgs),
    CmdGizmoDrawPolyline(gizmo::CmdGizmoDrawPolylineArgs),
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
    Log(galfus_log::LogEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
    NotificationSend(sys::CmdResultNotificationSend),
    SystemDiagnosticsSet(sys::CmdResultSystemDiagnosticsSet),
    SystemLogLevelSet(sys::CmdResultSystemLogLevelSet),
    SystemLogLevelGet(sys::CmdResultSystemLogLevelGet),
    SystemBuildVersionGet(sys::CmdResultSystemBuildVersionGet),
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowMeasurement(win::CmdResultWindowMeasurement),
    WindowCursor(win::CmdResultWindowCursor),
    WindowState(win::CmdResultWindowState),
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
    MaterialDefinitionUpsert(res::CmdResultMaterialDefinition),
    MaterialDefinitionDispose(res::CmdResultMaterialDefinition),
    MaterialDefinitionGet(res::CmdResultResourceGet),
    MaterialDefinitionList(res::CmdResultResourceList),
    MaterialInstanceUpsert(res::CmdResultMaterialInstance),
    MaterialInstanceDispose(res::CmdResultMaterialInstance),
    MaterialInstanceGet(res::CmdResultResourceGet),
    MaterialInstanceList(res::CmdResultResourceList),
    TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer),
    TextureCreateSolidColor(res::CmdResultTextureCreateSolidColor),
    TextureUpsert(CmdResultSimple),
    TextureDispose(res::CmdResultTextureDispose),
    TextureBindTarget(res::CmdResultTextureBindTarget),
    TextureGet(res::CmdResultResourceGet),
    AudioListenerUpsert(CmdResultSimple),
    AudioListenerDispose(audio::CmdResultAudioListenerDispose),
    AudioListenerGet(audio::CmdResultAudioListenerGet),
    AudioResourceUpsert(audio::CmdResultAudioResourceUpsert),
    AudioResourceGet(audio::CmdResultAudioResourceGet),
    AudioResourceList(audio::CmdResultAudioResourceList),
    AudioSourceUpsert(CmdResultSimple),
    AudioSourceGet(audio::CmdResultAudioSourceGet),
    AudioSourceList(audio::CmdResultAudioSourceList),
    AudioSourceTransport(audio::CmdResultAudioSourceTransport),
    AudioStateGet(audio::CmdResultAudioStateGet),
    AudioSourceDispose(audio::CmdResultAudioSourceDispose),
    AudioResourceDispose(audio::CmdResultAudioResourceDispose),
    GeometryUpsert(CmdResultSimple),
    GeometryDispose(res::CmdResultGeometryDispose),
    GeometryGet(res::CmdResultResourceGet),
    PrimitiveGeometryCreate(res::CmdResultPrimitiveGeometryCreate),
    EnvironmentUpsert(CmdResultSimple),
    EnvironmentDispose(res::CmdResultEnvironment),
    EnvironmentGet(res::CmdResultResourceGet),
    EnvironmentList(res::CmdResultResourceList),
    ShadowConfigure(res::shadow::CmdResultShadowConfigure),
    RealmCreate(realm::CmdResultRealmCreate),
    RealmDispose(realm::CmdResultRealmDispose),
    RealmGet(realm::CmdResultRealmGet),
    RealmList(realm::CmdResultRealmList),
    RenderGraphUpsert(realm::CmdResultRenderGraphUpsert),
    RenderGraphDispose(realm::CmdResultRenderGraphDispose),
    RenderGraphList(realm::CmdResultRenderGraphList),
    RealmRenderGraphBind(realm::CmdResultRealmRenderGraphBind),
    TargetUpsert(target::CmdResultTargetUpsert),
    TargetGet(target::CmdResultTargetGet),
    TargetList(target::CmdResultTargetList),
    TargetMeasurement(target::CmdResultTargetMeasurement),
    TargetDispose(target::CmdResultTargetDispose),
    TargetLayerUpsert(target::CmdResultTargetLayerUpsert),
    TargetLayerDispose(target::CmdResultTargetLayerDispose),
    TargetLayerGet(target::CmdResultTargetLayerGet),
    TargetLayerList(target::CmdResultTargetLayerList),
    ModelGet(res::CmdResultResourceGet),
    ModelList(res::CmdResultModelList),
    MaterialGet(res::CmdResultResourceGet),
    MaterialList(res::CmdResultMaterialList),
    TextureList(res::CmdResultTextureList),
    GeometryList(res::CmdResultGeometryList),
    LightGet(res::CmdResultResourceGet),
    LightList(res::CmdResultLightList),
    CameraGet(res::CmdResultResourceGet),
    CameraList(res::CmdResultCameraList),
    GizmoDrawLine(gizmo::CmdResultGizmoDraw),
    GizmoDrawAabb(gizmo::CmdResultGizmoDraw),
    GizmoDrawPolyline(gizmo::CmdResultGizmoDraw),
}

pub type EngineCmdEnvelope = galfus_protocol::CommandEnvelope<EngineCmd>;

pub type CommandResponseEnvelope = galfus_protocol::ResponseEnvelope<CommandResponse>;

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

mod processing;

pub(crate) use processing::{deferred_command_key, engine_process_batch};
