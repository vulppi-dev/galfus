use crate::core::render::passes::UiPlatformAction;
use crate::core::state::EngineState;

pub(super) fn apply_ui_platform_actions(
    engine_state: &mut EngineState,
    actions: Vec<UiPlatformAction>,
) {
    for action in actions {
        match action {
            UiPlatformAction::SetCursorIcon { window_id, icon } => {
                let _ = crate::core::window::engine_cmd_window_cursor_from_ui(
                    engine_state,
                    &crate::core::window::CmdWindowCursorArgs {
                        window_id,
                        icon: Some(icon),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::OpenUrl {
                window_id,
                realm_id,
                url,
                new_tab,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiOpenUrl {
                            window_id,
                            realm_id,
                            url,
                            new_tab,
                        },
                    ));
            }
            UiPlatformAction::ClipboardSetText {
                window_id,
                realm_id,
                text,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiClipboardSetText {
                            window_id,
                            realm_id,
                            text,
                        },
                    ));
            }
            UiPlatformAction::ClipboardRequestCopy {
                window_id,
                realm_id,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiClipboardRequestCopy {
                            window_id,
                            realm_id,
                        },
                    ));
            }
            UiPlatformAction::ClipboardRequestCut {
                window_id,
                realm_id,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiClipboardRequestCut {
                            window_id,
                            realm_id,
                        },
                    ));
            }
            UiPlatformAction::ClipboardRequestPaste {
                window_id,
                realm_id,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiClipboardRequestPaste {
                            window_id,
                            realm_id,
                        },
                    ));
            }
            UiPlatformAction::RequestFocus { window_id } => {
                #[cfg(not(feature = "wasm"))]
                let already_focused = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| window_state.window.has_focus())
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_focused = false;
                if already_focused {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        action: Some(crate::core::window::WindowStateAction::Focus),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::RequestAttention {
                window_id,
                attention,
            } => {
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        action: Some(crate::core::window::WindowStateAction::RequestAttention),
                        attention_type: attention,
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::ScreenshotRequest {
                window_id,
                realm_id,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiScreenshotRequest {
                            window_id,
                            realm_id,
                        },
                    ));
            }
            UiPlatformAction::SetWindowTitle { window_id, title } => {
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        title: Some(title),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::SetWindowSize {
                window_id,
                width,
                height,
            } => {
                let target_width = width.max(1);
                let target_height = height.max(1);
                #[cfg(not(feature = "wasm"))]
                let already_applied = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| {
                        let inner_size = window_state.window.inner_size();
                        inner_size.width.abs_diff(target_width) <= 1
                            && inner_size.height.abs_diff(target_height) <= 1
                    })
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_applied = false;
                if already_applied {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_measurement(
                    engine_state,
                    &crate::core::window::CmdWindowMeasurementArgs {
                        window_id,
                        size: Some(glam::UVec2::new(target_width, target_height)),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::SetWindowPosition { window_id, x, y } => {
                #[cfg(not(feature = "wasm"))]
                let already_applied = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| {
                        window_state.outer_position.x == x && window_state.outer_position.y == y
                    })
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_applied = false;
                if already_applied {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_measurement(
                    engine_state,
                    &crate::core::window::CmdWindowMeasurementArgs {
                        window_id,
                        position: Some(glam::IVec2::new(x, y)),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::SetWindowResizable { window_id, value } => {
                #[cfg(not(feature = "wasm"))]
                let already_applied = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| window_state.window.is_resizable() == value)
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_applied = false;
                if already_applied {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        resizable: Some(value),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::SetWindowDecorations { window_id, value } => {
                #[cfg(not(feature = "wasm"))]
                let already_applied = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| window_state.window.is_decorated() == value)
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_applied = false;
                if already_applied {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        decorations: Some(value),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::SetWindowState { window_id, state } => {
                #[cfg(not(feature = "wasm"))]
                let already_applied = engine_state
                    .window
                    .states
                    .get(&window_id)
                    .map(|window_state| current_window_state_for_ui(window_state) == state)
                    .unwrap_or(false);
                #[cfg(feature = "wasm")]
                let already_applied = false;
                if already_applied {
                    continue;
                }
                let _ = crate::core::window::engine_cmd_window_state(
                    engine_state,
                    &crate::core::window::CmdWindowStateArgs {
                        window_id,
                        state: Some(state),
                        ..Default::default()
                    },
                );
            }
            UiPlatformAction::EmitViewportSync {
                window_id,
                realm_id,
                viewport_id,
                parent_viewport_id,
                class,
                title,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiViewportSync {
                            window_id,
                            realm_id,
                            viewport_id,
                            parent_viewport_id,
                            class,
                            title,
                        },
                    ));
            }
            UiPlatformAction::EmitViewportCommand {
                window_id,
                realm_id,
                viewport_id,
                command,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiViewportCommand {
                            window_id,
                            realm_id,
                            viewport_id,
                            command,
                        },
                    ));
            }
            UiPlatformAction::EmitViewportFallbackEmbedded {
                window_id,
                realm_id,
                viewport_id,
                parent_viewport_id,
            } => {
                engine_state
                    .runtime
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::SystemEvent::UiViewportFallbackEmbedded {
                            window_id,
                            realm_id,
                            viewport_id,
                            parent_viewport_id,
                        },
                    ));
            }
        }
    }
}

#[cfg(not(feature = "wasm"))]
fn current_window_state_for_ui(
    window_state: &crate::core::window::WindowState,
) -> crate::core::window::EngineWindowState {
    let fullscreen = match window_state.window.fullscreen() {
        Some(crate::core::platform::winit::window::Fullscreen::Exclusive(_)) => {
            Some(vulfram_platform::PlatformFullscreenMode::Exclusive)
        }
        Some(crate::core::platform::winit::window::Fullscreen::Borderless(_)) => {
            Some(vulfram_platform::PlatformFullscreenMode::Borderless)
        }
        None => None,
    };

    match vulfram_platform::resolve_platform_window_state(
        window_state.window.is_minimized().unwrap_or(false),
        window_state.window.is_maximized(),
        fullscreen,
    ) {
        vulfram_platform::PlatformWindowLifecycleState::Windowed => {
            crate::core::window::EngineWindowState::Windowed
        }
        vulfram_platform::PlatformWindowLifecycleState::Fullscreen => {
            crate::core::window::EngineWindowState::Fullscreen
        }
        vulfram_platform::PlatformWindowLifecycleState::WindowedFullscreen => {
            crate::core::window::EngineWindowState::WindowedFullscreen
        }
        vulfram_platform::PlatformWindowLifecycleState::Maximized => {
            crate::core::window::EngineWindowState::Maximized
        }
        vulfram_platform::PlatformWindowLifecycleState::Minimized => {
            crate::core::window::EngineWindowState::Minimized
        }
    }
}
