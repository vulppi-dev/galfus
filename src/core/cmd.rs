use crate::core::platforms::PlatformProxy;
use serde::{Deserialize, Serialize};

use crate::core::VulframResult;
use crate::core::gamepad::events::GamepadEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::state::EngineState;
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
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowSetTitle(win::CmdWindowSetTitleArgs),
    CmdWindowSetPosition(win::CmdWindowSetPositionArgs),
    CmdWindowGetPosition(win::CmdWindowGetPositionArgs),
    CmdWindowSetSize(win::CmdWindowSetSizeArgs),
    CmdWindowGetSize(win::CmdWindowGetSizeArgs),
    CmdWindowGetOuterSize(win::CmdWindowGetOuterSizeArgs),
    CmdWindowGetSurfaceSize(win::CmdWindowGetSurfaceSizeArgs),
    CmdWindowSetState(win::CmdWindowSetStateArgs),
    CmdWindowGetState(win::CmdWindowGetStateArgs),
    CmdWindowSetIcon(win::CmdWindowSetIconArgs),
    CmdWindowSetDecorations(win::CmdWindowSetDecorationsArgs),
    CmdWindowHasDecorations(win::CmdWindowHasDecorationsArgs),
    CmdWindowSetResizable(win::CmdWindowSetResizableArgs),
    CmdWindowIsResizable(win::CmdWindowIsResizableArgs),
    CmdWindowRequestAttention(win::CmdWindowRequestAttentionArgs),
    CmdWindowFocus(win::CmdWindowFocusArgs),
    CmdWindowSetCursorVisible(win::CmdWindowSetCursorVisibleArgs),
    CmdWindowSetCursorGrab(win::CmdWindowSetCursorGrabArgs),
    CmdWindowSetCursorIcon(win::CmdWindowSetCursorIconArgs),
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
    CmdAudioResourceCreate(audio::CmdAudioResourceCreateArgs),
    CmdAudioResourcePush(audio::CmdAudioResourcePushArgs),
    CmdAudioSourceUpsert(CmdAudioSourceUpsertArgs),
    CmdAudioSourcePlay(audio::CmdAudioSourcePlayArgs),
    CmdAudioSourcePause(audio::CmdAudioSourcePauseArgs),
    CmdAudioSourceStop(audio::CmdAudioSourceStopArgs),
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
    CmdTargetUpsert(target::CmdTargetUpsertArgs),
    CmdTargetDispose(target::CmdTargetDisposeArgs),
    CmdTargetLayerUpsert(target::CmdTargetLayerUpsertArgs),
    CmdTargetLayerDispose(target::CmdTargetLayerDisposeArgs),
    CmdUiThemeDefine(ui::CmdUiThemeDefineArgs),
    CmdUiThemeDispose(ui::CmdUiThemeDisposeArgs),
    CmdUiDocumentCreate(ui::CmdUiDocumentCreateArgs),
    CmdUiDocumentDispose(ui::CmdUiDocumentDisposeArgs),
    CmdUiDocumentSetRect(ui::CmdUiDocumentSetRectArgs),
    CmdUiDocumentSetTheme(ui::CmdUiDocumentSetThemeArgs),
    CmdUiApplyOps(ui::CmdUiApplyOpsArgs),
    CmdUiDebugSet(ui::CmdUiDebugSetArgs),
    CmdUiImageCreateFromBuffer(ui::CmdUiImageCreateFromBufferArgs),
    CmdUiImageDispose(ui::CmdUiImageDisposeArgs),
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
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowSetTitle(win::CmdResultWindowSetTitle),
    WindowSetPosition(win::CmdResultWindowSetPosition),
    WindowGetPosition(win::CmdResultWindowGetPosition),
    WindowSetSize(win::CmdResultWindowSetSize),
    WindowGetSize(win::CmdResultWindowGetSize),
    WindowGetOuterSize(win::CmdResultWindowGetOuterSize),
    WindowGetSurfaceSize(win::CmdResultWindowGetSurfaceSize),
    WindowSetState(win::CmdResultWindowSetState),
    WindowGetState(win::CmdResultWindowGetState),
    WindowSetIcon(win::CmdResultWindowSetIcon),
    WindowSetDecorations(win::CmdResultWindowSetDecorations),
    WindowHasDecorations(win::CmdResultWindowHasDecorations),
    WindowSetResizable(win::CmdResultWindowSetResizable),
    WindowIsResizable(win::CmdResultWindowIsResizable),
    WindowRequestAttention(win::CmdResultWindowRequestAttention),
    WindowFocus(win::CmdResultWindowFocus),
    WindowSetCursorVisible(win::CmdResultWindowSetCursorVisible),
    WindowSetCursorGrab(win::CmdResultWindowSetCursorGrab),
    WindowSetCursorIcon(win::CmdResultWindowSetCursorIcon),
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
    AudioResourceCreate(audio::CmdResultAudioResourceCreate),
    AudioResourcePush(audio::CmdResultAudioResourcePush),
    AudioSourceUpsert(CmdResultSimple),
    AudioSourcePlay(audio::CmdResultAudioSourcePlay),
    AudioSourcePause(audio::CmdResultAudioSourcePause),
    AudioSourceStop(audio::CmdResultAudioSourceStop),
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
    TargetUpsert(target::CmdResultTargetUpsert),
    TargetDispose(target::CmdResultTargetDispose),
    TargetLayerUpsert(target::CmdResultTargetLayerUpsert),
    TargetLayerDispose(target::CmdResultTargetLayerDispose),
    UiThemeDefine(ui::CmdResultUiThemeDefine),
    UiThemeDispose(ui::CmdResultUiThemeDispose),
    UiDocumentCreate(ui::CmdResultUiDocumentCreate),
    UiDocumentDispose(ui::CmdResultUiDocumentDispose),
    UiDocumentSetRect(ui::CmdResultUiDocumentSetRect),
    UiDocumentSetTheme(ui::CmdResultUiDocumentSetTheme),
    UiApplyOps(ui::CmdResultUiApplyOps),
    UiDebugSet(ui::CmdResultUiDebugSet),
    UiImageCreateFromBuffer(ui::CmdResultUiImageCreateFromBuffer),
    UiImageDispose(ui::CmdResultUiImageDispose),
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

fn maybe_emit_response_error_event(
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
        CommandResponse::AudioResourceCreate(result) => {
            failure_case!(result, "audio-resource-create")
        }
        CommandResponse::AudioResourcePush(result) => failure_case!(result, "audio-resource-push"),
        CommandResponse::AudioSourceUpsert(result) => failure_case!(result, "audio-source-upsert"),
        CommandResponse::AudioSourcePlay(result) => failure_case!(result, "audio-source-play"),
        CommandResponse::AudioSourcePause(result) => failure_case!(result, "audio-source-pause"),
        CommandResponse::AudioSourceStop(result) => failure_case!(result, "audio-source-stop"),
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
        CommandResponse::UiApplyOps(result) => failure_case!(result, "ui-apply-ops"),
        CommandResponse::UiDebugSet(result) => failure_case!(result, "ui-debug-set"),
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

pub fn engine_process_batch(
    engine: &mut EngineState,
    platform: &mut dyn PlatformProxy,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        let response_count_before = engine.response_queue.len();
        let command_id = pack.id;
        match pack.cmd {
            EngineCmd::CmdNotificationSend(args) => {
                let result =
                    sys::engine_cmd_notification_send(engine, platform.event_loop_proxy(), &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::NotificationSend(result),
                });
            }
            EngineCmd::CmdSystemDiagnosticsSet(args) => {
                let result = sys::engine_cmd_system_diagnostics_set(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::SystemDiagnosticsSet(result),
                });
            }
            EngineCmd::CmdWindowCreate(args) => {
                match platform.handle_window_create(engine, pack.id, &args) {
                    Ok(()) => {}
                    Err(result) => {
                        engine.response_queue.push(CommandResponseEnvelope {
                            id: pack.id,
                            response: CommandResponse::WindowCreate(result),
                        });
                    }
                }
            }
            EngineCmd::CmdWindowClose(args) => {
                let result = win::engine_cmd_window_close(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowClose(result),
                });
            }
            EngineCmd::CmdWindowSetTitle(args) => {
                let result = win::engine_cmd_window_set_title(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetTitle(result),
                });
            }
            EngineCmd::CmdWindowSetPosition(args) => {
                let result = win::engine_cmd_window_set_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetPosition(result),
                });
            }
            EngineCmd::CmdWindowGetPosition(args) => {
                let result = win::engine_cmd_window_get_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetPosition(result),
                });
            }
            EngineCmd::CmdWindowSetSize(args) => {
                let result = win::engine_cmd_window_set_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetSize(result),
                });
            }
            EngineCmd::CmdWindowGetSize(args) => {
                let result = win::engine_cmd_window_get_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSize(result),
                });
            }
            EngineCmd::CmdWindowGetOuterSize(args) => {
                let result = win::engine_cmd_window_get_outer_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetOuterSize(result),
                });
            }
            EngineCmd::CmdWindowGetSurfaceSize(args) => {
                let result = win::engine_cmd_window_get_surface_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSurfaceSize(result),
                });
            }
            EngineCmd::CmdWindowSetState(args) => {
                let result = win::engine_cmd_window_set_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetState(result),
                });
            }
            EngineCmd::CmdWindowGetState(args) => {
                let result = win::engine_cmd_window_get_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetState(result),
                });
            }
            EngineCmd::CmdWindowSetIcon(args) => {
                let result = win::engine_cmd_window_set_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetIcon(result),
                });
            }
            EngineCmd::CmdWindowSetDecorations(args) => {
                let result = win::engine_cmd_window_set_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetDecorations(result),
                });
            }
            EngineCmd::CmdWindowHasDecorations(args) => {
                let result = win::engine_cmd_window_has_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowHasDecorations(result),
                });
            }
            EngineCmd::CmdWindowSetResizable(args) => {
                let result = win::engine_cmd_window_set_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetResizable(result),
                });
            }
            EngineCmd::CmdWindowIsResizable(args) => {
                let result = win::engine_cmd_window_is_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowIsResizable(result),
                });
            }
            EngineCmd::CmdWindowRequestAttention(args) => {
                let result = win::engine_cmd_window_request_attention(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowRequestAttention(result),
                });
            }
            EngineCmd::CmdWindowFocus(args) => {
                let result = win::engine_cmd_window_focus(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowFocus(result),
                });
            }
            EngineCmd::CmdWindowSetCursorVisible(args) => {
                let result = win::engine_cmd_window_set_cursor_visible(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorVisible(result),
                });
            }
            EngineCmd::CmdWindowSetCursorGrab(args) => {
                let result = win::engine_cmd_window_set_cursor_grab(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorGrab(result),
                });
            }
            EngineCmd::CmdWindowSetCursorIcon(args) => {
                let result = win::engine_cmd_window_set_cursor_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorIcon(result),
                });
            }
            EngineCmd::CmdUploadBufferDiscardAll(args) => {
                let result = buf::engine_cmd_upload_buffer_discard_all(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UploadBufferDiscardAll(result),
                });
            }
            EngineCmd::CmdCameraUpsert(args) => {
                let result = match args {
                    CmdCameraUpsertArgs::Create(create_args) => {
                        let create_result = res::engine_cmd_camera_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdCameraUpsertArgs::Update(update_args) => {
                        let update_result = res::engine_cmd_camera_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraUpsert(result),
                });
            }
            EngineCmd::CmdCameraDispose(args) => {
                let result = res::engine_cmd_camera_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraDispose(result),
                });
            }
            EngineCmd::CmdModelUpsert(args) => {
                let result = match args {
                    CmdModelUpsertArgs::Create(create_args) => {
                        let create_result = res::engine_cmd_model_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdModelUpsertArgs::Update(update_args) => {
                        let update_result = res::engine_cmd_model_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelUpsert(result),
                });
            }
            EngineCmd::CmdPoseUpdate(args) => {
                let result = res::engine_cmd_pose_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::PoseUpdate(result),
                });
            }
            EngineCmd::CmdModelDispose(args) => {
                let result = res::engine_cmd_model_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelDispose(result),
                });
            }
            EngineCmd::CmdLightUpsert(args) => {
                let result = match args {
                    CmdLightUpsertArgs::Create(create_args) => {
                        let create_result = res::engine_cmd_light_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdLightUpsertArgs::Update(update_args) => {
                        let update_result = res::engine_cmd_light_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightUpsert(result),
                });
            }
            EngineCmd::CmdLightDispose(args) => {
                let result = res::engine_cmd_light_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightDispose(result),
                });
            }
            EngineCmd::CmdMaterialUpsert(args) => {
                let result = match args {
                    CmdMaterialUpsertArgs::Create(create_args) => {
                        let create_result = res::engine_cmd_material_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdMaterialUpsertArgs::Update(update_args) => {
                        let update_result = res::engine_cmd_material_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialUpsert(result),
                });
            }
            EngineCmd::CmdMaterialDispose(args) => {
                let result = res::engine_cmd_material_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialDispose(result),
                });
            }
            EngineCmd::CmdTextureCreateFromBuffer(args) => {
                let result = res::engine_cmd_texture_create_from_buffer(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureCreateFromBuffer(result),
                });
            }
            EngineCmd::CmdTextureCreateSolidColor(args) => {
                let result = res::engine_cmd_texture_create_solid_color(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureCreateSolidColor(result),
                });
            }
            EngineCmd::CmdTextureDispose(args) => {
                let result = res::engine_cmd_texture_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureDispose(result),
                });
            }
            EngineCmd::CmdTextureBindTarget(args) => {
                let result = res::engine_cmd_texture_bind_target(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureBindTarget(result),
                });
            }
            EngineCmd::CmdAudioListenerUpsert(args) => {
                let result = match args {
                    CmdAudioListenerUpsertArgs::Create(create_args) => {
                        let create_result =
                            audio::engine_cmd_audio_listener_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdAudioListenerUpsertArgs::Update(update_args) => {
                        let update_result =
                            audio::engine_cmd_audio_listener_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioListenerUpsert(result),
                });
            }
            EngineCmd::CmdAudioListenerDispose(args) => {
                let result = audio::engine_cmd_audio_listener_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioListenerDispose(result),
                });
            }
            EngineCmd::CmdAudioResourceCreate(args) => {
                let result = audio::engine_cmd_audio_buffer_create_from_buffer(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioResourceCreate(result),
                });
            }
            EngineCmd::CmdAudioResourcePush(args) => {
                let result = audio::engine_cmd_audio_resource_push(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioResourcePush(result),
                });
            }
            EngineCmd::CmdAudioSourceUpsert(args) => {
                let result = match args {
                    CmdAudioSourceUpsertArgs::Create(create_args) => {
                        let create_result =
                            audio::engine_cmd_audio_source_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdAudioSourceUpsertArgs::Update(update_args) => {
                        let update_result =
                            audio::engine_cmd_audio_source_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioSourceUpsert(result),
                });
            }
            EngineCmd::CmdAudioSourcePlay(args) => {
                let result = audio::engine_cmd_audio_source_play(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioSourcePlay(result),
                });
            }
            EngineCmd::CmdAudioSourcePause(args) => {
                let result = audio::engine_cmd_audio_source_pause(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioSourcePause(result),
                });
            }
            EngineCmd::CmdAudioSourceStop(args) => {
                let result = audio::engine_cmd_audio_source_stop(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioSourceStop(result),
                });
            }
            EngineCmd::CmdAudioSourceDispose(args) => {
                let result = audio::engine_cmd_audio_source_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioSourceDispose(result),
                });
            }
            EngineCmd::CmdAudioResourceDispose(args) => {
                let result = audio::engine_cmd_audio_resource_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::AudioResourceDispose(result),
                });
            }
            EngineCmd::CmdGeometryUpsert(args) => {
                let result = match args {
                    CmdGeometryUpsertArgs::Create(create_args) => {
                        let create_result = res::engine_cmd_geometry_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdGeometryUpsertArgs::Update(update_args) => {
                        let update_result = res::engine_cmd_geometry_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryUpsert(result),
                });
            }
            EngineCmd::CmdGeometryDispose(args) => {
                let result = res::engine_cmd_geometry_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryDispose(result),
                });
            }
            EngineCmd::CmdPrimitiveGeometryCreate(args) => {
                let result = res::engine_cmd_primitive_geometry_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::PrimitiveGeometryCreate(result),
                });
            }
            EngineCmd::CmdEnvironmentUpsert(args) => {
                let result = match args {
                    CmdEnvironmentUpsertArgs::Create(create_args) => {
                        let create_result =
                            res::engine_cmd_environment_create(engine, &create_args);
                        CmdResultSimple {
                            success: create_result.success,
                            message: create_result.message,
                        }
                    }
                    CmdEnvironmentUpsertArgs::Update(update_args) => {
                        let update_result =
                            res::engine_cmd_environment_update(engine, &update_args);
                        CmdResultSimple {
                            success: update_result.success,
                            message: update_result.message,
                        }
                    }
                };
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::EnvironmentUpsert(result),
                });
            }
            EngineCmd::CmdEnvironmentDispose(args) => {
                let result = res::engine_cmd_environment_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::EnvironmentDispose(result),
                });
            }
            EngineCmd::CmdShadowConfigure(args) => {
                let result = res::shadow::engine_cmd_shadow_configure(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ShadowConfigure(result),
                });
            }
            EngineCmd::CmdRealmCreate(args) => {
                let result = realm::engine_cmd_realm_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::RealmCreate(result),
                });
            }
            EngineCmd::CmdRealmDispose(args) => {
                let result = realm::engine_cmd_realm_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::RealmDispose(result),
                });
            }
            EngineCmd::CmdTargetUpsert(args) => {
                let result = target::engine_cmd_target_upsert(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TargetUpsert(result),
                });
            }
            EngineCmd::CmdTargetDispose(args) => {
                let result = target::engine_cmd_target_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TargetDispose(result),
                });
            }
            EngineCmd::CmdTargetLayerUpsert(args) => {
                let result = target::engine_cmd_target_layer_upsert(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TargetLayerUpsert(result),
                });
            }
            EngineCmd::CmdTargetLayerDispose(args) => {
                let result = target::engine_cmd_target_layer_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TargetLayerDispose(result),
                });
            }
            EngineCmd::CmdUiThemeDefine(args) => {
                let result = ui::engine_cmd_ui_theme_define(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiThemeDefine(result),
                });
            }
            EngineCmd::CmdUiThemeDispose(args) => {
                let result = ui::engine_cmd_ui_theme_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiThemeDispose(result),
                });
            }
            EngineCmd::CmdUiDocumentCreate(args) => {
                let result = ui::engine_cmd_ui_document_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiDocumentCreate(result),
                });
            }
            EngineCmd::CmdUiDocumentDispose(args) => {
                let result = ui::engine_cmd_ui_document_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiDocumentDispose(result),
                });
            }
            EngineCmd::CmdUiDocumentSetRect(args) => {
                let result = ui::engine_cmd_ui_document_set_rect(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiDocumentSetRect(result),
                });
            }
            EngineCmd::CmdUiDocumentSetTheme(args) => {
                let result = ui::engine_cmd_ui_document_set_theme(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiDocumentSetTheme(result),
                });
            }
            EngineCmd::CmdUiApplyOps(args) => {
                let result = ui::engine_cmd_ui_apply_ops(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiApplyOps(result),
                });
            }
            EngineCmd::CmdUiDebugSet(args) => {
                let result = ui::engine_cmd_ui_debug_set(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiDebugSet(result),
                });
            }
            EngineCmd::CmdUiImageCreateFromBuffer(args) => {
                let result = ui::engine_cmd_ui_image_create_from_buffer(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiImageCreateFromBuffer(result),
                });
            }
            EngineCmd::CmdUiImageDispose(args) => {
                let result = ui::engine_cmd_ui_image_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UiImageDispose(result),
                });
            }
            EngineCmd::CmdModelList(args) => {
                let result = res::engine_cmd_model_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelList(result),
                });
            }
            EngineCmd::CmdMaterialList(args) => {
                let result = res::engine_cmd_material_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialList(result),
                });
            }
            EngineCmd::CmdTextureList(args) => {
                let result = res::engine_cmd_texture_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureList(result),
                });
            }
            EngineCmd::CmdGeometryList(args) => {
                let result = res::engine_cmd_geometry_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryList(result),
                });
            }
            EngineCmd::CmdLightList(args) => {
                let result = res::engine_cmd_light_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightList(result),
                });
            }
            EngineCmd::CmdCameraList(args) => {
                let result = res::engine_cmd_camera_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraList(result),
                });
            }
            EngineCmd::CmdGizmoDrawLine(args) => {
                for window_state in engine.window.states.values_mut() {
                    window_state
                        .render_state
                        .gizmos
                        .add_line(args.start, args.end, args.color);
                    window_state.is_dirty = true;
                }
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GizmoDrawLine(gizmo::CmdResultGizmoDraw {
                        status: 0,
                    }),
                });
            }
            EngineCmd::CmdGizmoDrawAabb(args) => {
                for window_state in engine.window.states.values_mut() {
                    window_state
                        .render_state
                        .gizmos
                        .add_aabb(args.min, args.max, args.color);
                    window_state.is_dirty = true;
                }
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GizmoDrawAabb(gizmo::CmdResultGizmoDraw {
                        status: 0,
                    }),
                });
            }
        }
        if engine.response_queue.len() > response_count_before
            && let Some(last_response) = engine.response_queue.last().cloned()
        {
            maybe_emit_response_error_event(engine, command_id, &last_response.response);
        }
    }

    VulframResult::Success
}
