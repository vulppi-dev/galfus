use serde::{Deserialize, Serialize};

#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CursorGrabMode {
    #[default]
    None = 0,
    Confined,
    Locked,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCursorArgs {
    pub window_id: u32,
    pub visible: Option<bool>,
    pub mode: Option<CursorGrabMode>,
    pub icon: Option<CursorIcon>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCursor {
    pub success: bool,
    pub message: String,
}

#[cfg(not(feature = "wasm"))]
fn map_cursor_icon(icon: CursorIcon) -> winit::window::CursorIcon {
    match icon {
        CursorIcon::Default => winit::window::CursorIcon::Default,
        CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
        CursorIcon::Help => winit::window::CursorIcon::Help,
        CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
        CursorIcon::Progress => winit::window::CursorIcon::Progress,
        CursorIcon::Wait => winit::window::CursorIcon::Wait,
        CursorIcon::Cell => winit::window::CursorIcon::Cell,
        CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
        CursorIcon::Text => winit::window::CursorIcon::Text,
        CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
        CursorIcon::Alias => winit::window::CursorIcon::Alias,
        CursorIcon::Copy => winit::window::CursorIcon::Copy,
        CursorIcon::Move => winit::window::CursorIcon::Move,
        CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
        CursorIcon::Grab => winit::window::CursorIcon::Grab,
        CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
        CursorIcon::EResize => winit::window::CursorIcon::EResize,
        CursorIcon::NResize => winit::window::CursorIcon::NResize,
        CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
        CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
        CursorIcon::SResize => winit::window::CursorIcon::SResize,
        CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
        CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
        CursorIcon::WResize => winit::window::CursorIcon::WResize,
        CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
        CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
        CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
        CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
        CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
        CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
        CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
        CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
        CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
    }
}

#[cfg(not(feature = "wasm"))]
fn apply_window_cursor(
    engine: &mut EngineState,
    args: &CmdWindowCursorArgs,
    persist_icon_override: bool,
) -> CmdResultWindowCursor {
    let Some(window_state) = engine.window.states.get(&args.window_id) else {
        return CmdResultWindowCursor {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    };

    if let Some(visible) = args.visible {
        window_state.window.set_cursor_visible(visible);
    }

    if let Some(mode) = args.mode {
        let raw_mode = match mode {
            CursorGrabMode::None => winit::window::CursorGrabMode::None,
            CursorGrabMode::Confined => winit::window::CursorGrabMode::Confined,
            CursorGrabMode::Locked => winit::window::CursorGrabMode::Locked,
        };

        if let Err(error) = window_state.window.set_cursor_grab(raw_mode) {
            return CmdResultWindowCursor {
                success: false,
                message: format!("Failed to set cursor grab mode: {:?}", error),
            };
        }
    }

    if let Some(icon) = args.icon {
        window_state.window.set_cursor(map_cursor_icon(icon));
        if persist_icon_override {
            engine
                .window
                .cursor_icon_override
                .insert(args.window_id, icon);
        }
    }

    CmdResultWindowCursor {
        success: true,
        message: "Window cursor command applied successfully".into(),
    }
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_cursor(
    engine: &mut EngineState,
    args: &CmdWindowCursorArgs,
) -> CmdResultWindowCursor {
    apply_window_cursor(engine, args, true)
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_cursor_from_ui(
    engine: &mut EngineState,
    args: &CmdWindowCursorArgs,
) -> CmdResultWindowCursor {
    if let Some(override_icon) = engine.window.cursor_icon_override.get(&args.window_id) {
        if let Some(window_state) = engine.window.states.get(&args.window_id) {
            window_state
                .window
                .set_cursor(map_cursor_icon(*override_icon));
            return CmdResultWindowCursor {
                success: true,
                message: "Window cursor UI update ignored due to host override".into(),
            };
        }
    }
    apply_window_cursor(engine, args, false)
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_cursor(
    _engine: &mut EngineState,
    args: &CmdWindowCursorArgs,
) -> CmdResultWindowCursor {
    CmdResultWindowCursor {
        success: false,
        message: format!(
            "Window cursor commands are not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_cursor_from_ui(
    _engine: &mut EngineState,
    args: &CmdWindowCursorArgs,
) -> CmdResultWindowCursor {
    CmdResultWindowCursor {
        success: false,
        message: format!(
            "Window cursor commands are not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}
