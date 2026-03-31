use crate::core::cmd::EngineEvent;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit;
use crate::core::state::EngineState;
use crate::core::window::{WindowEvent, WindowPointerCaptureState};
pub use vulfram_protocol::{
    CmdResultWindowCursor, CmdWindowCursorArgs, CursorGrabMode, CursorIcon,
};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen::JsCast;

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
    let Some(window) = engine
        .window
        .states
        .get(&args.window_id)
        .map(|window_state| window_state.window.clone())
    else {
        return CmdResultWindowCursor {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    };

    if let Some(visible) = args.visible {
        window.set_cursor_visible(visible);
    }

    if let Some(mode) = args.mode {
        let raw_mode = match mode {
            CursorGrabMode::None => winit::window::CursorGrabMode::None,
            CursorGrabMode::Confined => winit::window::CursorGrabMode::Confined,
            CursorGrabMode::Locked => winit::window::CursorGrabMode::Locked,
        };

        if let Err(error) = window.set_cursor_grab(raw_mode) {
            return CmdResultWindowCursor {
                success: false,
                message: format!("Failed to set cursor grab mode: {:?}", error),
            };
        }
        engine.window.set_cursor_grab_mode(args.window_id, mode);
        let active = mode != CursorGrabMode::None;
        engine
            .window
            .set_pointer_capture_active(args.window_id, active);
        engine
            .runtime
            .event_queue
            .push(EngineEvent::Window(WindowEvent::OnPointerCaptureChange {
                window_id: args.window_id,
                capture: WindowPointerCaptureState {
                    mode,
                    active,
                    reason: Some("command".into()),
                },
            }));
    }

    if let Some(icon) = args.icon {
        window.set_cursor(map_cursor_icon(icon));
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
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window_state) = _engine.window.states.get(&args.window_id) else {
            return CmdResultWindowCursor {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        };

        let Some(mode) = args.mode else {
            if args.visible.is_some() || args.icon.is_some() {
                return CmdResultWindowCursor {
                    success: false,
                    message: "WASM cursor currently supports only mode updates".into(),
                };
            }
            return CmdResultWindowCursor {
                success: true,
                message: "No cursor mode changes applied".into(),
            };
        };

        let window_ref = window_state.window.clone();
        let canvas = window_ref.canvas();
        _engine.window.set_cursor_grab_mode(args.window_id, mode);

        match mode {
            CursorGrabMode::None => {
                if let Some(window) = web_sys::window()
                    && let Some(document) = window.document()
                {
                    document.exit_pointer_lock();
                }
                _engine
                    .window
                    .set_pointer_capture_active(args.window_id, false);
                _engine
                    .runtime
                    .runtime
                    .event_queue
                    .push(EngineEvent::Window(WindowEvent::OnPointerCaptureChange {
                        window_id: args.window_id,
                        capture: WindowPointerCaptureState {
                            mode,
                            active: false,
                            reason: Some("command".into()),
                        },
                    }));
            }
            CursorGrabMode::Confined => {
                // Browser has no native confined mode; we emulate it logically in pointer input.
                _engine
                    .window
                    .set_pointer_capture_active(args.window_id, true);
                _engine
                    .runtime
                    .runtime
                    .event_queue
                    .push(EngineEvent::Window(WindowEvent::OnPointerCaptureChange {
                        window_id: args.window_id,
                        capture: WindowPointerCaptureState {
                            mode,
                            active: true,
                            reason: Some("command-polyfill".into()),
                        },
                    }));
            }
            CursorGrabMode::Locked => {
                let element: &web_sys::Element = canvas.unchecked_ref();
                element.request_pointer_lock();
                _engine
                    .window
                    .set_pointer_capture_active(args.window_id, false);
                _engine
                    .runtime
                    .runtime
                    .event_queue
                    .push(EngineEvent::Window(WindowEvent::OnPointerCaptureChange {
                        window_id: args.window_id,
                        capture: WindowPointerCaptureState {
                            mode,
                            active: false,
                            reason: Some("command-requested".into()),
                        },
                    }));
            }
        }

        return CmdResultWindowCursor {
            success: true,
            message: match mode {
                CursorGrabMode::None => "Pointer capture disabled".into(),
                CursorGrabMode::Confined => "Pointer confined mode enabled (polyfill)".into(),
                CursorGrabMode::Locked => "Pointer lock requested".into(),
            },
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
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
    engine_cmd_window_cursor(_engine, args)
}
