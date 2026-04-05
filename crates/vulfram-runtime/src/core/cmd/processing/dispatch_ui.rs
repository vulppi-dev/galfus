use super::super::*;
use crate::core::state::EngineState;
use crate::core::ui::cmd::{
    CmdUiAccessKitActionRequestArgs, CmdUiApplyOpsArgs, CmdUiClipboardPasteArgs, CmdUiDebugSetArgs,
    CmdUiDocumentCreateArgs, CmdUiDocumentDisposeArgs, CmdUiDocumentGetLayoutRectsArgs,
    CmdUiDocumentGetTreeArgs, CmdUiDocumentSetRectArgs, CmdUiDocumentSetThemeArgs,
    CmdUiEventTraceSetArgs, CmdUiFocusGetArgs, CmdUiFocusSetArgs, CmdUiImageCreateFromBufferArgs,
    CmdUiImageDisposeArgs, CmdUiScreenshotReplyArgs, CmdUiThemeDefineArgs, CmdUiThemeDisposeArgs,
};

fn mark_windows_dirty(engine: &mut EngineState) {
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
}

pub(super) fn cmd_ui_theme_define(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiThemeDefineArgs,
) {
    let result = ui::engine_cmd_ui_theme_define(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiThemeDefine(result),
    });
}

pub(super) fn cmd_ui_theme_dispose(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiThemeDisposeArgs,
) {
    let result = ui::engine_cmd_ui_theme_dispose(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiThemeDispose(result),
    });
}

pub(super) fn cmd_ui_document_create(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentCreateArgs,
) {
    let result = ui::engine_cmd_ui_document_create(engine, &args);
    if result.success {
        mark_windows_dirty(engine);
    }
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentCreate(result),
    });
}

pub(super) fn cmd_ui_document_dispose(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentDisposeArgs,
) {
    let result = ui::engine_cmd_ui_document_dispose(engine, &args);
    if result.success {
        mark_windows_dirty(engine);
    }
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentDispose(result),
    });
}

pub(super) fn cmd_ui_document_set_rect(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentSetRectArgs,
) {
    let result = ui::engine_cmd_ui_document_set_rect(engine, &args);
    if result.success {
        mark_windows_dirty(engine);
    }
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentSetRect(result),
    });
}

pub(super) fn cmd_ui_document_set_theme(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentSetThemeArgs,
) {
    let result = ui::engine_cmd_ui_document_set_theme(engine, &args);
    if result.success {
        mark_windows_dirty(engine);
    }
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentSetTheme(result),
    });
}

pub(super) fn cmd_ui_document_get_tree(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentGetTreeArgs,
) {
    let result = ui::engine_cmd_ui_document_get_tree(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentGetTree(result),
    });
}

pub(super) fn cmd_ui_document_get_layout_rects(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiDocumentGetLayoutRectsArgs,
) {
    let result = ui::engine_cmd_ui_document_get_layout_rects(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDocumentGetLayoutRects(result),
    });
}

pub(super) fn cmd_ui_apply_ops(engine: &mut EngineState, command_id: u64, args: CmdUiApplyOpsArgs) {
    let result = ui::engine_cmd_ui_apply_ops(engine, &args);
    if result.success {
        mark_windows_dirty(engine);
    }
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiApplyOps(result),
    });
}

pub(super) fn cmd_ui_debug_set(engine: &mut EngineState, command_id: u64, args: CmdUiDebugSetArgs) {
    let result = ui::engine_cmd_ui_debug_set(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiDebugSet(result),
    });
}

pub(super) fn cmd_ui_focus_set(engine: &mut EngineState, command_id: u64, args: CmdUiFocusSetArgs) {
    let result = ui::engine_cmd_ui_focus_set(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiFocusSet(result),
    });
}

pub(super) fn cmd_ui_focus_get(engine: &mut EngineState, command_id: u64, args: CmdUiFocusGetArgs) {
    let result = ui::engine_cmd_ui_focus_get(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiFocusGet(result),
    });
}

pub(super) fn cmd_ui_event_trace_set(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiEventTraceSetArgs,
) {
    let result = ui::engine_cmd_ui_event_trace_set(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiEventTraceSet(result),
    });
}

pub(super) fn cmd_ui_image_create_from_buffer(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiImageCreateFromBufferArgs,
) {
    let result = ui::engine_cmd_ui_image_create_from_buffer(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiImageCreateFromBuffer(result),
    });
}

pub(super) fn cmd_ui_image_dispose(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiImageDisposeArgs,
) {
    let result = ui::engine_cmd_ui_image_dispose(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiImageDispose(result),
    });
}

pub(super) fn cmd_ui_clipboard_paste(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiClipboardPasteArgs,
) {
    let result = ui::engine_cmd_ui_clipboard_paste(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiClipboardPaste(result),
    });
}

pub(super) fn cmd_ui_screenshot_reply(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiScreenshotReplyArgs,
) {
    let result = ui::engine_cmd_ui_screenshot_reply(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiScreenshotReply(result),
    });
}

pub(super) fn cmd_ui_accesskit_action_request(
    engine: &mut EngineState,
    command_id: u64,
    args: CmdUiAccessKitActionRequestArgs,
) {
    let result = ui::engine_cmd_ui_accesskit_action_request(engine, &args);
    engine.runtime.push_response(CommandResponseEnvelope {
        id: command_id,
        response: CommandResponse::UiAccessKitActionRequest(result),
    });
}
