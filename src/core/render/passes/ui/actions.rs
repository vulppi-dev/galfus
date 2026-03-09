use std::hash::{Hash, Hasher};

use crate::core::realm::RealmId;
use crate::core::system::{UiViewportClass, UiViewportCommand};
use crate::core::window::{CursorIcon, EngineWindowState, UserAttentionType};

#[derive(Debug, Clone)]
pub enum UiPlatformAction {
    SetCursorIcon {
        window_id: u32,
        icon: CursorIcon,
    },
    OpenUrl {
        window_id: u32,
        realm_id: u32,
        url: String,
        new_tab: bool,
    },
    ClipboardSetText {
        window_id: u32,
        realm_id: u32,
        text: String,
    },
    ClipboardRequestCopy {
        window_id: u32,
        realm_id: u32,
    },
    ClipboardRequestCut {
        window_id: u32,
        realm_id: u32,
    },
    ClipboardRequestPaste {
        window_id: u32,
        realm_id: u32,
    },
    RequestFocus {
        window_id: u32,
    },
    RequestAttention {
        window_id: u32,
        attention: Option<UserAttentionType>,
    },
    ScreenshotRequest {
        window_id: u32,
        realm_id: u32,
    },
    SetWindowTitle {
        window_id: u32,
        title: String,
    },
    SetWindowSize {
        window_id: u32,
        width: u32,
        height: u32,
    },
    SetWindowPosition {
        window_id: u32,
        x: i32,
        y: i32,
    },
    SetWindowResizable {
        window_id: u32,
        value: bool,
    },
    SetWindowDecorations {
        window_id: u32,
        value: bool,
    },
    SetWindowState {
        window_id: u32,
        state: EngineWindowState,
    },
    EmitViewportSync {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
        class: UiViewportClass,
        title: Option<String>,
    },
    EmitViewportCommand {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        command: UiViewportCommand,
    },
    EmitViewportFallbackEmbedded {
        window_id: u32,
        realm_id: u32,
        viewport_id: u64,
        parent_viewport_id: Option<u64>,
    },
}

pub fn collect_platform_actions(
    output: &egui::FullOutput,
    window_id: u32,
    realm_id: RealmId,
) -> Vec<UiPlatformAction> {
    let mut actions = Vec::new();
    if let Some(icon) = map_cursor_icon(output.platform_output.cursor_icon) {
        actions.push(UiPlatformAction::SetCursorIcon { window_id, icon });
    }
    if let Some(open_url) = output.platform_output.open_url.as_ref() {
        actions.push(UiPlatformAction::OpenUrl {
            window_id,
            realm_id: realm_id.0,
            url: open_url.url.clone(),
            new_tab: open_url.new_tab,
        });
    }
    if !output.platform_output.copied_text.is_empty() {
        actions.push(UiPlatformAction::ClipboardSetText {
            window_id,
            realm_id: realm_id.0,
            text: output.platform_output.copied_text.clone(),
        });
    }
    for (viewport_id, viewport) in &output.viewport_output {
        let viewport_id = viewport_id_key(*viewport_id);
        let parent_viewport_id = Some(viewport_id_key(viewport.parent));
        actions.push(UiPlatformAction::EmitViewportSync {
            window_id,
            realm_id: realm_id.0,
            viewport_id,
            parent_viewport_id,
            class: map_viewport_class(viewport.class),
            title: viewport.builder.title.clone(),
        });
        if !matches!(viewport.class, egui::ViewportClass::Root) {
            actions.push(UiPlatformAction::EmitViewportFallbackEmbedded {
                window_id,
                realm_id: realm_id.0,
                viewport_id,
                parent_viewport_id,
            });
        }
        for command in &viewport.commands {
            if !matches!(viewport.class, egui::ViewportClass::Root) {
                if let Some(command) = map_viewport_command(command) {
                    actions.push(UiPlatformAction::EmitViewportCommand {
                        window_id,
                        realm_id: realm_id.0,
                        viewport_id,
                        command,
                    });
                }
                continue;
            }
            match command {
                egui::ViewportCommand::Title(title) => {
                    actions.push(UiPlatformAction::SetWindowTitle {
                        window_id,
                        title: title.clone(),
                    });
                }
                egui::ViewportCommand::InnerSize(size) => {
                    let ppp = output.pixels_per_point.max(0.001);
                    actions.push(UiPlatformAction::SetWindowSize {
                        window_id,
                        width: (size.x * ppp).max(1.0).round() as u32,
                        height: (size.y * ppp).max(1.0).round() as u32,
                    });
                }
                egui::ViewportCommand::OuterPosition(pos) => {
                    actions.push(UiPlatformAction::SetWindowPosition {
                        window_id,
                        x: pos.x.round() as i32,
                        y: pos.y.round() as i32,
                    });
                }
                egui::ViewportCommand::Resizable(value) => {
                    actions.push(UiPlatformAction::SetWindowResizable {
                        window_id,
                        value: *value,
                    });
                }
                egui::ViewportCommand::Decorations(value) => {
                    actions.push(UiPlatformAction::SetWindowDecorations {
                        window_id,
                        value: *value,
                    });
                }
                egui::ViewportCommand::Fullscreen(value) => {
                    if *value {
                        actions.push(UiPlatformAction::SetWindowState {
                            window_id,
                            state: EngineWindowState::Fullscreen,
                        });
                    }
                }
                egui::ViewportCommand::Minimized(value) => {
                    if *value {
                        actions.push(UiPlatformAction::SetWindowState {
                            window_id,
                            state: EngineWindowState::Minimized,
                        });
                    }
                }
                egui::ViewportCommand::Maximized(value) => {
                    if *value {
                        actions.push(UiPlatformAction::SetWindowState {
                            window_id,
                            state: EngineWindowState::Maximized,
                        });
                    }
                }
                egui::ViewportCommand::RequestCopy => {
                    actions.push(UiPlatformAction::ClipboardRequestCopy {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::RequestCut => {
                    actions.push(UiPlatformAction::ClipboardRequestCut {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::RequestPaste => {
                    actions.push(UiPlatformAction::ClipboardRequestPaste {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                egui::ViewportCommand::Focus => {
                    actions.push(UiPlatformAction::RequestFocus { window_id });
                }
                egui::ViewportCommand::RequestUserAttention(attention) => {
                    actions.push(UiPlatformAction::RequestAttention {
                        window_id,
                        attention: map_attention_type(*attention),
                    });
                }
                egui::ViewportCommand::Screenshot => {
                    actions.push(UiPlatformAction::ScreenshotRequest {
                        window_id,
                        realm_id: realm_id.0,
                    });
                }
                _ => {
                    if let Some(command) = map_viewport_command(command) {
                        actions.push(UiPlatformAction::EmitViewportCommand {
                            window_id,
                            realm_id: realm_id.0,
                            viewport_id,
                            command,
                        });
                    }
                }
            }
        }
    }
    actions
}

fn map_attention_type(attention: egui::UserAttentionType) -> Option<UserAttentionType> {
    match attention {
        egui::UserAttentionType::Critical => Some(UserAttentionType::Critical),
        egui::UserAttentionType::Informational => Some(UserAttentionType::Informational),
        egui::UserAttentionType::Reset => None,
    }
}

fn map_viewport_class(class: egui::ViewportClass) -> UiViewportClass {
    match class {
        egui::ViewportClass::Root => UiViewportClass::Root,
        egui::ViewportClass::Deferred => UiViewportClass::Deferred,
        egui::ViewportClass::Immediate => UiViewportClass::Immediate,
        egui::ViewportClass::Embedded => UiViewportClass::Embedded,
    }
}

fn viewport_id_key(viewport_id: egui::ViewportId) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    viewport_id.hash(&mut hasher);
    hasher.finish()
}

fn map_viewport_command(command: &egui::ViewportCommand) -> Option<UiViewportCommand> {
    match command {
        egui::ViewportCommand::Close => Some(UiViewportCommand::Close),
        egui::ViewportCommand::Title(title) => Some(UiViewportCommand::Title {
            title: title.clone(),
        }),
        egui::ViewportCommand::InnerSize(size) => Some(UiViewportCommand::InnerSize {
            width: size.x,
            height: size.y,
        }),
        egui::ViewportCommand::OuterPosition(pos) => {
            Some(UiViewportCommand::OuterPosition { x: pos.x, y: pos.y })
        }
        egui::ViewportCommand::Resizable(value) => {
            Some(UiViewportCommand::Resizable { value: *value })
        }
        egui::ViewportCommand::Decorations(value) => {
            Some(UiViewportCommand::Decorations { value: *value })
        }
        egui::ViewportCommand::Fullscreen(value) => {
            Some(UiViewportCommand::Fullscreen { value: *value })
        }
        egui::ViewportCommand::Minimized(value) => {
            Some(UiViewportCommand::Minimized { value: *value })
        }
        egui::ViewportCommand::Maximized(value) => {
            Some(UiViewportCommand::Maximized { value: *value })
        }
        egui::ViewportCommand::Focus => Some(UiViewportCommand::Focus),
        egui::ViewportCommand::Screenshot => Some(UiViewportCommand::Screenshot),
        egui::ViewportCommand::CursorVisible(value) => {
            Some(UiViewportCommand::CursorVisible { value: *value })
        }
        egui::ViewportCommand::CursorGrab(mode) => Some(UiViewportCommand::CursorGrab {
            mode: format!("{:?}", mode),
        }),
        egui::ViewportCommand::IMEAllowed(value) => {
            Some(UiViewportCommand::ImeAllowed { value: *value })
        }
        egui::ViewportCommand::IMERect(rect) => Some(UiViewportCommand::ImeRect {
            min_x: rect.min.x,
            min_y: rect.min.y,
            max_x: rect.max.x,
            max_y: rect.max.y,
        }),
        _ => None,
    }
}

fn map_cursor_icon(icon: egui::CursorIcon) -> Option<CursorIcon> {
    match icon {
        egui::CursorIcon::None => None,
        egui::CursorIcon::Default => Some(CursorIcon::Default),
        egui::CursorIcon::ContextMenu => Some(CursorIcon::ContextMenu),
        egui::CursorIcon::Help => Some(CursorIcon::Help),
        egui::CursorIcon::PointingHand => Some(CursorIcon::Pointer),
        egui::CursorIcon::Progress => Some(CursorIcon::Progress),
        egui::CursorIcon::Wait => Some(CursorIcon::Wait),
        egui::CursorIcon::Cell => Some(CursorIcon::Cell),
        egui::CursorIcon::Crosshair => Some(CursorIcon::Crosshair),
        egui::CursorIcon::Text => Some(CursorIcon::Text),
        egui::CursorIcon::VerticalText => Some(CursorIcon::VerticalText),
        egui::CursorIcon::Alias => Some(CursorIcon::Alias),
        egui::CursorIcon::Copy => Some(CursorIcon::Copy),
        egui::CursorIcon::Move => Some(CursorIcon::Move),
        egui::CursorIcon::NoDrop => Some(CursorIcon::NoDrop),
        egui::CursorIcon::NotAllowed => Some(CursorIcon::NotAllowed),
        egui::CursorIcon::Grab => Some(CursorIcon::Grab),
        egui::CursorIcon::Grabbing => Some(CursorIcon::Grabbing),
        egui::CursorIcon::AllScroll => Some(CursorIcon::AllScroll),
        egui::CursorIcon::ResizeHorizontal => Some(CursorIcon::EwResize),
        egui::CursorIcon::ResizeNeSw => Some(CursorIcon::NeswResize),
        egui::CursorIcon::ResizeNwSe => Some(CursorIcon::NwseResize),
        egui::CursorIcon::ResizeVertical => Some(CursorIcon::NsResize),
        egui::CursorIcon::ResizeEast => Some(CursorIcon::EResize),
        egui::CursorIcon::ResizeSouthEast => Some(CursorIcon::SeResize),
        egui::CursorIcon::ResizeSouth => Some(CursorIcon::SResize),
        egui::CursorIcon::ResizeSouthWest => Some(CursorIcon::SwResize),
        egui::CursorIcon::ResizeWest => Some(CursorIcon::WResize),
        egui::CursorIcon::ResizeNorthWest => Some(CursorIcon::NwResize),
        egui::CursorIcon::ResizeNorth => Some(CursorIcon::NResize),
        egui::CursorIcon::ResizeNorthEast => Some(CursorIcon::NeResize),
        egui::CursorIcon::ResizeColumn => Some(CursorIcon::ColResize),
        egui::CursorIcon::ResizeRow => Some(CursorIcon::RowResize),
        egui::CursorIcon::ZoomIn => Some(CursorIcon::ZoomIn),
        egui::CursorIcon::ZoomOut => Some(CursorIcon::ZoomOut),
    }
}
