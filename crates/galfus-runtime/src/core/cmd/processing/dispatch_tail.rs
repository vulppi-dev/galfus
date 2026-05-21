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
        EngineCmd::CmdModelGet(args) => {
            let result = res::engine_cmd_model_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::ModelGet(result),
            });
        }
        EngineCmd::CmdMaterialGet(args) => {
            let result = res::engine_cmd_material_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialGet(result),
            });
        }
        EngineCmd::CmdTextureGet(args) => {
            let result = res::engine_cmd_texture_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TextureGet(result),
            });
        }
        EngineCmd::CmdGeometryGet(args) => {
            let result = res::engine_cmd_geometry_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::GeometryGet(result),
            });
        }
        EngineCmd::CmdLightGet(args) => {
            let result = res::engine_cmd_light_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::LightGet(result),
            });
        }
        EngineCmd::CmdCameraGet(args) => {
            let result = res::engine_cmd_camera_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::CameraGet(result),
            });
        }
        EngineCmd::CmdEnvironmentGet(args) => {
            let result = res::engine_cmd_environment_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::EnvironmentGet(result),
            });
        }
        EngineCmd::CmdEnvironmentList(args) => {
            let result = res::engine_cmd_environment_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::EnvironmentList(result),
            });
        }
        EngineCmd::CmdMaterialDefinitionGet(args) => {
            let result = res::engine_cmd_material_definition_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialDefinitionGet(result),
            });
        }
        EngineCmd::CmdMaterialDefinitionList(args) => {
            let result = res::engine_cmd_material_definition_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialDefinitionList(result),
            });
        }
        EngineCmd::CmdMaterialInstanceGet(args) => {
            let result = res::engine_cmd_material_instance_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialInstanceGet(result),
            });
        }
        EngineCmd::CmdMaterialInstanceList(args) => {
            let result = res::engine_cmd_material_instance_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::MaterialInstanceList(result),
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
        EngineCmd::CmdAudioListenerGet(args) => {
            let result = audio::engine_cmd_audio_listener_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::AudioListenerGet(result),
            });
        }
        EngineCmd::CmdAudioSourceGet(args) => {
            let result = audio::engine_cmd_audio_source_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::AudioSourceGet(result),
            });
        }
        EngineCmd::CmdAudioSourceList(args) => {
            let result = audio::engine_cmd_audio_source_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::AudioSourceList(result),
            });
        }
        EngineCmd::CmdAudioResourceGet(args) => {
            let result = audio::engine_cmd_audio_resource_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::AudioResourceGet(result),
            });
        }
        EngineCmd::CmdAudioResourceList(args) => {
            let result = audio::engine_cmd_audio_resource_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::AudioResourceList(result),
            });
        }
        EngineCmd::CmdRealmGet(args) => {
            let result = realm::engine_cmd_realm_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::RealmGet(result),
            });
        }
        EngineCmd::CmdRealmList(args) => {
            let result = realm::engine_cmd_realm_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::RealmList(result),
            });
        }
        EngineCmd::CmdTargetGet(args) => {
            let result = target::engine_cmd_target_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TargetGet(result),
            });
        }
        EngineCmd::CmdTargetList(args) => {
            let result = target::engine_cmd_target_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TargetList(result),
            });
        }
        EngineCmd::CmdTargetLayerGet(args) => {
            let result = target::engine_cmd_target_layer_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TargetLayerGet(result),
            });
        }
        EngineCmd::CmdTargetLayerList(args) => {
            let result = target::engine_cmd_target_layer_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: envelope_id,
                response: CommandResponse::TargetLayerList(result),
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
