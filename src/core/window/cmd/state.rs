#[cfg(not(feature = "wasm"))]
use crate::core::buffers::state::UploadType;
#[cfg(not(feature = "wasm"))]
use crate::core::image::ImageDecoder;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit;
use serde::{Deserialize, Serialize};

use crate::core::cmd::EngineEvent;
use crate::core::state::EngineState;
#[cfg(not(feature = "wasm"))]
use crate::core::system::push_error_event;
use crate::core::window::WindowEvent;

use super::EngineWindowState;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum UserAttentionType {
    Critical = 0,
    Informational,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum WindowStateAction {
    Focus,
    RequestAttention,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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

#[cfg(not(feature = "wasm"))]
fn set_window_icon(engine: &mut EngineState, window_id: u32, buffer_id: u64) -> Result<(), String> {
    if !engine.window.states.contains_key(&window_id) {
        return Err(format!("Window with id {} not found", window_id));
    }

    let Some(buffer) = engine.buffers.remove_upload(buffer_id) else {
        return Err(format!("Buffer with id {} not found", buffer_id));
    };

    if buffer.upload_type != UploadType::ImageData {
        return Err(format!(
            "Invalid buffer type. Expected ImageData, got {:?}",
            buffer.upload_type
        ));
    }

    let Some(image_buffer) = ImageDecoder::try_decode(&buffer.data) else {
        return Err(
            "Failed to decode image. Supported formats: PNG, JPEG, WebP, AVIF, EXR, HDR".into(),
        );
    };

    let image_data = match image_buffer.pixels {
        crate::core::image::ImagePixels::Rgba8(data) => data,
        crate::core::image::ImagePixels::Rgba16F(_) => {
            return Err("Window icon requires RGBA8 image data".into());
        }
    };

    let icon = winit::window::Icon::from_rgba(image_data, image_buffer.width, image_buffer.height)
        .map_err(|e| format!("Failed to create icon: {:?}", e))?;

    let Some(window_state) = engine.window.states.get(&window_id) else {
        return Err(format!("Window with id {} not found", window_id));
    };

    window_state.window.set_window_icon(Some(icon));
    Ok(())
}

#[cfg(not(feature = "wasm"))]
fn read_window_state(window_state: &crate::core::window::WindowState) -> EngineWindowState {
    if window_state.window.is_minimized().unwrap_or(false) {
        EngineWindowState::Minimized
    } else if window_state.window.is_maximized() {
        EngineWindowState::Maximized
    } else if window_state.window.fullscreen().is_some() {
        match window_state.window.fullscreen() {
            Some(winit::window::Fullscreen::Exclusive(_)) => EngineWindowState::Fullscreen,
            Some(winit::window::Fullscreen::Borderless(_)) => EngineWindowState::WindowedFullscreen,
            None => EngineWindowState::Windowed,
        }
    } else {
        EngineWindowState::Windowed
    }
}

#[cfg(not(feature = "wasm"))]
fn apply_window_state(
    window_state: &crate::core::window::WindowState,
    state: EngineWindowState,
) -> Result<(), String> {
    match state {
        EngineWindowState::Minimized => {
            window_state.window.set_minimized(true);
        }
        EngineWindowState::Maximized => {
            window_state.window.set_maximized(true);
        }
        EngineWindowState::Windowed => {
            window_state.window.set_minimized(false);
            window_state.window.set_maximized(false);
            window_state.window.set_fullscreen(None);
        }
        EngineWindowState::Fullscreen => {
            let monitor = window_state.window.current_monitor();
            let exclusive = monitor
                .as_ref()
                .and_then(|current_monitor| current_monitor.video_modes().next())
                .map(winit::window::Fullscreen::Exclusive);
            if let Some(fullscreen) = exclusive {
                window_state.window.set_fullscreen(Some(fullscreen));
            } else {
                return Err("Failed to set fullscreen: no exclusive video mode available".into());
            }
        }
        EngineWindowState::WindowedFullscreen => {
            let monitor = window_state.window.current_monitor();
            let fullscreen = Some(winit::window::Fullscreen::Borderless(monitor));
            window_state.window.set_fullscreen(fullscreen);
        }
    }
    Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_state(
    engine: &mut EngineState,
    args: &CmdWindowStateArgs,
) -> CmdResultWindowState {
    let Some(window_state) = engine.window.states.get(&args.window_id) else {
        return CmdResultWindowState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            ..Default::default()
        };
    };

    if let Some(title) = args.title.as_ref() {
        window_state.window.set_title(title);
    }

    if let Some(state) = args.state {
        if let Err(message) = apply_window_state(window_state, state) {
            push_error_event(
                engine,
                "window-state",
                message.clone(),
                None,
                Some("window-state".into()),
            );
            return CmdResultWindowState {
                success: false,
                message,
                ..Default::default()
            };
        }
    }

    if let Some(decorations) = args.decorations {
        window_state.window.set_decorations(decorations);
    }

    if let Some(resizable) = args.resizable {
        window_state.window.set_resizable(resizable);
    }

    if let Some(action) = args.action {
        match action {
            WindowStateAction::Focus => {
                window_state.window.focus_window();
            }
            WindowStateAction::RequestAttention => {
                let attention_type = args.attention_type.map(|t| match t {
                    UserAttentionType::Critical => winit::window::UserAttentionType::Critical,
                    UserAttentionType::Informational => {
                        winit::window::UserAttentionType::Informational
                    }
                });
                window_state.window.request_user_attention(attention_type);
            }
        }
    }

    if let Some(buffer_id) = args.icon_buffer_id {
        if let Err(message) = set_window_icon(engine, args.window_id, buffer_id) {
            return CmdResultWindowState {
                success: false,
                message,
                ..Default::default()
            };
        }
    }

    let read_state = args.get_state || args.state.is_some();
    let read_decorations = args.get_decorations || args.decorations.is_some();
    let read_resizable = args.get_resizable || args.resizable.is_some();

    let Some(window_state) = engine.window.states.get(&args.window_id) else {
        return CmdResultWindowState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            ..Default::default()
        };
    };
    let current_state = read_window_state(window_state);
    let current_decorations = window_state.window.is_decorated();
    let current_resizable = window_state.window.is_resizable();
    if engine
        .window
        .set_lifecycle_state(args.window_id, current_state)
    {
        engine
            .event_queue
            .push(EngineEvent::Window(WindowEvent::OnStateChange {
                window_id: args.window_id,
                state: current_state,
            }));
    }

    CmdResultWindowState {
        success: true,
        message: "Window state command applied successfully".into(),
        state: read_state.then_some(current_state),
        decorations: read_decorations.then_some(current_decorations),
        resizable: read_resizable.then_some(current_resizable),
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub fn engine_cmd_window_state(
    _engine: &mut EngineState,
    args: &CmdWindowStateArgs,
) -> CmdResultWindowState {
    let has_mutation = args.title.is_some()
        || args.state.is_some()
        || args.icon_buffer_id.is_some()
        || args.decorations.is_some()
        || args.resizable.is_some()
        || args.action.is_some()
        || args.attention_type.is_some();
    if has_mutation {
        return CmdResultWindowState {
            success: false,
            message: format!(
                "Window state mutation is not supported in wasm (window_id={})",
                args.window_id
            ),
            ..Default::default()
        };
    }
    if !args.get_state && !args.get_decorations && !args.get_resizable {
        return CmdResultWindowState {
            success: true,
            message: "No wasm window state getters requested".into(),
            ..Default::default()
        };
    }
    if !_engine.window.states.contains_key(&args.window_id) {
        return CmdResultWindowState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            ..Default::default()
        };
    }

    let Some(window) = web_sys::window() else {
        return CmdResultWindowState {
            success: false,
            message: "Web window not available".into(),
            ..Default::default()
        };
    };
    let Some(document) = window.document() else {
        return CmdResultWindowState {
            success: false,
            message: "Document not available".into(),
            ..Default::default()
        };
    };

    let lifecycle_state = if document.fullscreen_element().is_some() {
        EngineWindowState::Fullscreen
    } else {
        EngineWindowState::Windowed
    };
    if _engine
        .window
        .set_lifecycle_state(args.window_id, lifecycle_state)
    {
        _engine
            .event_queue
            .push(EngineEvent::Window(WindowEvent::OnStateChange {
                window_id: args.window_id,
                state: lifecycle_state,
            }));
    }

    let mut warnings = Vec::new();
    if args.get_decorations {
        warnings.push("decorations unavailable on canvas");
    }

    CmdResultWindowState {
        success: true,
        message: if warnings.is_empty() {
            "WASM window state getters applied".into()
        } else {
            format!(
                "WASM window state getters applied ({}).",
                warnings.join(", ")
            )
        },
        state: args.get_state.then_some(lifecycle_state),
        decorations: None,
        resizable: args.get_resizable.then_some(true),
    }
}

#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
pub fn engine_cmd_window_state(
    _engine: &mut EngineState,
    args: &CmdWindowStateArgs,
) -> CmdResultWindowState {
    CmdResultWindowState {
        success: false,
        message: format!(
            "Window state commands are not supported in wasm (window_id={})",
            args.window_id
        ),
        ..Default::default()
    }
}
