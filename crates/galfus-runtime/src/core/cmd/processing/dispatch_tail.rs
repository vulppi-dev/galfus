use super::super::*;
use crate::core::state::EngineState;

pub(super) fn dispatch_ui_and_misc(engine: &mut EngineState, envelope_id: u64, cmd: EngineCmd) {
    match cmd {
        EngineCmd::CmdModelList(args) => {
            let result = res::engine_cmd_model_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::ModelList(result),
            });
        }
        EngineCmd::CmdMaterialList(args) => {
            let result = res::engine_cmd_material_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialList(result),
            });
        }
        EngineCmd::CmdTextureList(args) => {
            let result = res::engine_cmd_texture_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TextureList(result),
            });
        }
        EngineCmd::CmdGeometryList(args) => {
            let result = res::engine_cmd_geometry_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GeometryList(result),
            });
        }
        EngineCmd::CmdLightList(args) => {
            let result = res::engine_cmd_light_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::LightList(result),
            });
        }
        EngineCmd::CmdCameraList(args) => {
            let result = res::engine_cmd_camera_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
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
            engine.runtime.push_response(CommandResponseEnvelope {
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
            engine.runtime.push_response(CommandResponseEnvelope {
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
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GizmoDrawPolyline(gizmo::CmdResultGizmoDraw {
                    status: 0,
                }),
            });
        }
        _ => {}
    }
}
