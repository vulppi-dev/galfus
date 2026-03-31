use super::super::*;
use crate::core::state::EngineState;

pub(super) fn dispatch_ui_and_misc(engine: &mut EngineState, envelope_id: u64, cmd: EngineCmd) {
    match cmd {
        EngineCmd::CmdUiThemeDefine(args) => {
            super::dispatch_ui::cmd_ui_theme_define(engine, envelope_id, args);
        }
        EngineCmd::CmdUiThemeDispose(args) => {
            super::dispatch_ui::cmd_ui_theme_dispose(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentCreate(args) => {
            super::dispatch_ui::cmd_ui_document_create(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentDispose(args) => {
            super::dispatch_ui::cmd_ui_document_dispose(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentSetRect(args) => {
            super::dispatch_ui::cmd_ui_document_set_rect(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentSetTheme(args) => {
            super::dispatch_ui::cmd_ui_document_set_theme(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentGetTree(args) => {
            super::dispatch_ui::cmd_ui_document_get_tree(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDocumentGetLayoutRects(args) => {
            super::dispatch_ui::cmd_ui_document_get_layout_rects(engine, envelope_id, args);
        }
        EngineCmd::CmdUiApplyOps(args) => {
            super::dispatch_ui::cmd_ui_apply_ops(engine, envelope_id, args);
        }
        EngineCmd::CmdUiDebugSet(args) => {
            super::dispatch_ui::cmd_ui_debug_set(engine, envelope_id, args);
        }
        EngineCmd::CmdUiFocusSet(args) => {
            super::dispatch_ui::cmd_ui_focus_set(engine, envelope_id, args);
        }
        EngineCmd::CmdUiFocusGet(args) => {
            super::dispatch_ui::cmd_ui_focus_get(engine, envelope_id, args);
        }
        EngineCmd::CmdUiEventTraceSet(args) => {
            super::dispatch_ui::cmd_ui_event_trace_set(engine, envelope_id, args);
        }
        EngineCmd::CmdUiImageCreateFromBuffer(args) => {
            super::dispatch_ui::cmd_ui_image_create_from_buffer(engine, envelope_id, args);
        }
        EngineCmd::CmdUiImageDispose(args) => {
            super::dispatch_ui::cmd_ui_image_dispose(engine, envelope_id, args);
        }
        EngineCmd::CmdUiClipboardPaste(args) => {
            super::dispatch_ui::cmd_ui_clipboard_paste(engine, envelope_id, args);
        }
        EngineCmd::CmdUiScreenshotReply(args) => {
            super::dispatch_ui::cmd_ui_screenshot_reply(engine, envelope_id, args);
        }
        EngineCmd::CmdUiAccessKitActionRequest(args) => {
            super::dispatch_ui::cmd_ui_accesskit_action_request(engine, envelope_id, args);
        }
        EngineCmd::CmdModelList(args) => {
            let result = res::engine_cmd_model_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::ModelList(result),
            });
        }
        EngineCmd::CmdMaterialList(args) => {
            let result = res::engine_cmd_material_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialList(result),
            });
        }
        EngineCmd::CmdTextureList(args) => {
            let result = res::engine_cmd_texture_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TextureList(result),
            });
        }
        EngineCmd::CmdGeometryList(args) => {
            let result = res::engine_cmd_geometry_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GeometryList(result),
            });
        }
        EngineCmd::CmdLightList(args) => {
            let result = res::engine_cmd_light_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::LightList(result),
            });
        }
        EngineCmd::CmdCameraList(args) => {
            let result = res::engine_cmd_camera_list(engine, &args);
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::CameraList(result),
            });
        }
        EngineCmd::CmdGizmoDrawLine(args) => {
            for (window_id, render_state) in engine.render.states.iter_mut() {
                let thickness = args.thickness.unwrap_or(0.0).max(0.0);
                render_state
                    .gizmos
                    .add_line(args.start, args.end, args.color, thickness);
                if let Some(window_state) = engine.window.states.get_mut(window_id) {
                    window_state.is_dirty = true;
                }
            }
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GizmoDrawLine(gizmo::CmdResultGizmoDraw { status: 0 }),
            });
        }
        EngineCmd::CmdGizmoDrawAabb(args) => {
            for (window_id, render_state) in engine.render.states.iter_mut() {
                let thickness = args.thickness.unwrap_or(0.0).max(0.0);
                render_state
                    .gizmos
                    .add_aabb(args.min, args.max, args.color, thickness);
                if let Some(window_state) = engine.window.states.get_mut(window_id) {
                    window_state.is_dirty = true;
                }
            }
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GizmoDrawAabb(gizmo::CmdResultGizmoDraw { status: 0 }),
            });
        }
        EngineCmd::CmdGizmoDrawPolyline(args) => {
            for (window_id, render_state) in engine.render.states.iter_mut() {
                let thickness = args.thickness.unwrap_or(0.0).max(0.0);
                render_state
                    .gizmos
                    .add_polyline(&args.points, args.color, args.closed, thickness);
                if let Some(window_state) = engine.window.states.get_mut(window_id) {
                    window_state.is_dirty = true;
                }
            }
            engine.runtime.response_queue.push(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GizmoDrawPolyline(gizmo::CmdResultGizmoDraw {
                    status: 0,
                }),
            });
        }
        _ => {}
    }
}
