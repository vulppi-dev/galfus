use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::system::push_error_event;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiClipboardPasteArgs {
    pub window_id: u32,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiScreenshotReplyArgs {
    pub window_id: u32,
    #[serde(default)]
    pub realm_id: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiAccessKitActionRequestArgs {
    pub window_id: u32,
    #[serde(default)]
    pub realm_id: Option<u32>,
    pub action: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiInputEvent {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_clipboard_paste(
    engine: &mut EngineState,
    args: &CmdUiClipboardPasteArgs,
) -> CmdResultUiInputEvent {
    let Some(realm_id) = engine
        .universal_state
        .ui
        .focus_by_window
        .get(&args.window_id)
        .copied()
    else {
        return CmdResultUiInputEvent {
            success: false,
            message: format!("No focused UI realm for window {}", args.window_id),
        };
    };
    if let Some(realm) = engine.universal_state.ui.realm_mut(realm_id) {
        realm.push_event(egui::Event::Paste(args.text.clone()));
        return CmdResultUiInputEvent {
            success: true,
            message: "UI paste event delivered".into(),
        };
    }
    CmdResultUiInputEvent {
        success: false,
        message: format!("UI realm {} not found", realm_id.0),
    }
}

pub fn engine_cmd_ui_screenshot_reply(
    engine: &mut EngineState,
    args: &CmdUiScreenshotReplyArgs,
) -> CmdResultUiInputEvent {
    let realm_id = if let Some(realm_id) = args.realm_id {
        crate::core::realm::RealmId(realm_id)
    } else {
        match engine
            .universal_state
            .ui
            .focus_by_window
            .get(&args.window_id)
            .copied()
        {
            Some(realm_id) => realm_id,
            None => {
                return CmdResultUiInputEvent {
                    success: false,
                    message: format!("No focused UI realm for window {}", args.window_id),
                };
            }
        }
    };

    let expected = args.width as usize * args.height as usize * 4;
    if args.rgba.len() != expected {
        return CmdResultUiInputEvent {
            success: false,
            message: format!(
                "Invalid screenshot payload size: expected {} bytes, got {}",
                expected,
                args.rgba.len()
            ),
        };
    }

    let image = egui::ColorImage::from_rgba_unmultiplied(
        [args.width as usize, args.height as usize],
        &args.rgba,
    );
    if let Some(realm) = engine.universal_state.ui.realm_mut(realm_id) {
        realm.push_event(egui::Event::Screenshot {
            viewport_id: egui::ViewportId::ROOT,
            image: Arc::new(image),
        });
        return CmdResultUiInputEvent {
            success: true,
            message: "UI screenshot reply delivered".into(),
        };
    }

    CmdResultUiInputEvent {
        success: false,
        message: format!("UI realm {} not found", realm_id.0),
    }
}

pub fn engine_cmd_ui_accesskit_action_request(
    engine: &mut EngineState,
    args: &CmdUiAccessKitActionRequestArgs,
) -> CmdResultUiInputEvent {
    let message = format!(
        "AccessKit action request fallback: not supported in current runtime (window={}, realm={:?}, action={})",
        args.window_id, args.realm_id, args.action
    );
    push_error_event(
        engine,
        "ui-input",
        message.clone(),
        None,
        Some("ui-accesskit-action-request".into()),
    );
    CmdResultUiInputEvent {
        success: false,
        message,
    }
}
