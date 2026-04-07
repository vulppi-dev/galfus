use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::{Display, Formatter};

pub use vulfram_types as types;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCodecError {
    message: String,
}

impl ProtocolCodecError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ProtocolCodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ProtocolCodecError {}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSimple {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdNotificationSendArgs {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
    pub level: NotificationLevel,
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultNotificationSend {
    pub success: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSystemBuildVersionGetArgs {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultSystemBuildVersionGet {
    pub success: bool,
    pub message: String,
    pub build_version: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EngineWindowState {
    Minimized = 0,
    Maximized,
    #[default]
    Windowed,
    Fullscreen,
    WindowedFullscreen,
}

pub fn window_size_default() -> glam::UVec2 {
    glam::UVec2::new(800, 600)
}

fn window_resizable_default() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CursorGrabMode {
    #[default]
    None = 0,
    Confined,
    Locked,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CursorIcon {
    #[default]
    Default = 0,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UserAttentionType {
    Critical = 0,
    Informational,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiViewportClass {
    Root,
    Deferred,
    Immediate,
    Embedded,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiViewportCommand {
    Close,
    Title {
        title: String,
    },
    InnerSize {
        width: f32,
        height: f32,
    },
    OuterPosition {
        x: f32,
        y: f32,
    },
    Resizable {
        value: bool,
    },
    Decorations {
        value: bool,
    },
    Fullscreen {
        value: bool,
    },
    Minimized {
        value: bool,
    },
    Maximized {
        value: bool,
    },
    Focus,
    Screenshot,
    CursorVisible {
        value: bool,
    },
    CursorGrab {
        mode: String,
    },
    ImeAllowed {
        value: bool,
    },
    ImeRect {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WindowStateAction {
    Focus,
    RequestAttention,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WindowPointerCaptureState {
    pub mode: CursorGrabMode,
    pub active: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum WindowEvent {
    #[serde(rename_all = "camelCase")]
    OnCreate { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnResize {
        window_id: u32,
        width: u32,
        height: u32,
    },
    #[serde(rename_all = "camelCase")]
    OnMove {
        window_id: u32,
        position: glam::IVec2,
    },
    #[serde(rename_all = "camelCase")]
    OnCloseRequest { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnDestroy { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnFocus { window_id: u32, focused: bool },
    #[serde(rename_all = "camelCase")]
    OnScaleFactorChange {
        window_id: u32,
        scale_factor: f64,
        new_width: u32,
        new_height: u32,
    },
    #[serde(rename_all = "camelCase")]
    OnOcclude { window_id: u32, occluded: bool },
    #[serde(rename_all = "camelCase")]
    OnRedrawRequest { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnFileDrop {
        window_id: u32,
        path: String,
        position: glam::Vec2,
    },
    #[serde(rename_all = "camelCase")]
    OnFileHover {
        window_id: u32,
        path: String,
        position: glam::Vec2,
    },
    #[serde(rename_all = "camelCase")]
    OnFileHoverCancel { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnThemeChange { window_id: u32, dark_mode: bool },
    #[serde(rename_all = "camelCase")]
    OnStateChange {
        window_id: u32,
        state: EngineWindowState,
    },
    #[serde(rename_all = "camelCase")]
    OnPointerCaptureChange {
        window_id: u32,
        capture: WindowPointerCaptureState,
    },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCloseArgs {
    pub window_id: u32,
}

pub type CmdResultWindowClose = CmdResultSimple;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub window_id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: glam::UVec2,
    #[serde(default)]
    pub position: glam::IVec2,
    #[serde(default)]
    pub canvas_id: Option<String>,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default = "window_resizable_default")]
    pub resizable: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub realm_id: Option<u32>,
    #[serde(default)]
    pub surface_id: Option<u32>,
    #[serde(default)]
    pub present_id: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowMeasurementArgs {
    pub window_id: u32,
    pub position: Option<glam::IVec2>,
    pub size: Option<glam::UVec2>,
    pub get_position: bool,
    pub get_size: bool,
    pub get_outer_size: bool,
    pub get_surface_size: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowMeasurement {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub position: Option<glam::IVec2>,
    #[serde(default)]
    pub size: Option<glam::UVec2>,
    #[serde(default)]
    pub outer_size: Option<glam::UVec2>,
    #[serde(default)]
    pub surface_size: Option<glam::UVec2>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCursorArgs {
    pub window_id: u32,
    pub visible: Option<bool>,
    pub mode: Option<CursorGrabMode>,
    pub icon: Option<CursorIcon>,
}

pub type CmdResultWindowCursor = CmdResultSimple;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowStateArgs {
    pub window_id: u32,
    pub title: Option<String>,
    pub state: Option<EngineWindowState>,
    pub icon_buffer_id: Option<u64>,
    pub decorations: Option<bool>,
    pub resizable: Option<bool>,
    pub action: Option<WindowStateAction>,
    pub attention_type: Option<UserAttentionType>,
    pub get_state: bool,
    pub get_decorations: bool,
    pub get_resizable: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowState {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub state: Option<EngineWindowState>,
    #[serde(default)]
    pub decorations: Option<bool>,
    #[serde(default)]
    pub resizable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope<T> {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEnvelope<T> {
    pub id: u64,
    #[serde(flatten)]
    pub response: T,
}

pub fn decode_named<T>(data: &[u8]) -> Result<T, ProtocolCodecError>
where
    T: DeserializeOwned,
{
    let mut deserializer = rmp_serde::Deserializer::new(data);
    serde_path_to_error::deserialize::<_, T>(&mut deserializer).map_err(|error| {
        let path = error.path().to_string();
        let inner = error.into_inner();
        if path.is_empty() {
            ProtocolCodecError::new(format!("invalid MessagePack payload: {inner}"))
        } else {
            ProtocolCodecError::new(format!("invalid MessagePack payload at '{path}': {inner}"))
        }
    })
}

pub fn encode_named<T>(value: &T) -> Result<Vec<u8>, ProtocolCodecError>
where
    T: Serialize,
{
    rmp_serde::to_vec_named(value)
        .map_err(|error| ProtocolCodecError::new(format!("failed to encode payload: {error}")))
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
