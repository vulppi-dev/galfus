use crate::core::audio::AudioSourceParams;
use crate::core::id_policy::validate_host_logical_id;
use crate::core::state::EngineState;
use galfus_audio::{
    AudioModelTransform, dispose_source, insert_source, plan_bound_source_updates, update_source,
};

use super::types::{
    AudioPlayModeDto, AudioSourceTransportActionDto, CmdAudioSourceCreateArgs,
    CmdAudioSourceDisposeArgs, CmdAudioSourceTransportArgs, CmdAudioSourceUpdateArgs,
    CmdResultAudioSourceCreate, CmdResultAudioSourceDispose, CmdResultAudioSourceTransport,
    CmdResultAudioSourceUpdate, audio_disabled_message,
};

pub fn engine_cmd_audio_source_create(
    engine: &mut EngineState,
    args: &CmdAudioSourceCreateArgs,
) -> CmdResultAudioSourceCreate {
    if let Err(message) = validate_host_logical_id(args.source_id, "sourceId") {
        return CmdResultAudioSourceCreate {
            success: false,
            message,
        };
    }
    if let Err(message) = validate_host_logical_id(args.realm_id, "realmId") {
        return CmdResultAudioSourceCreate {
            success: false,
            message,
        };
    }
    if let Err(message) = validate_host_logical_id(args.model_id, "modelId") {
        return CmdResultAudioSourceCreate {
            success: false,
            message,
        };
    }
    if !engine.audio_available {
        return CmdResultAudioSourceCreate {
            success: false,
            message: audio_disabled_message(),
        };
    }
    let params = AudioSourceParams {
        position: args.position,
        velocity: args.velocity,
        orientation: args.orientation,
        gain: args.gain,
        pitch: args.pitch,
        spatial: args.spatial.clone().into(),
    };

    insert_source(
        &mut engine.audio_state,
        args.source_id,
        args.realm_id,
        args.model_id,
        params,
    );
    match engine.audio.source_create(args.source_id, params) {
        Ok(()) => CmdResultAudioSourceCreate {
            success: true,
            message: "Source created".into(),
        },
        Err(message) => CmdResultAudioSourceCreate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_update(
    engine: &mut EngineState,
    args: &CmdAudioSourceUpdateArgs,
) -> CmdResultAudioSourceUpdate {
    if let Err(message) = validate_host_logical_id(args.source_id, "sourceId") {
        return CmdResultAudioSourceUpdate {
            success: false,
            message,
        };
    }
    if let Some(realm_id) = args.realm_id
        && let Err(message) = validate_host_logical_id(realm_id, "realmId")
    {
        return CmdResultAudioSourceUpdate {
            success: false,
            message,
        };
    }
    if let Some(model_id) = args.model_id
        && let Err(message) = validate_host_logical_id(model_id, "modelId")
    {
        return CmdResultAudioSourceUpdate {
            success: false,
            message,
        };
    }
    if !engine.audio_available {
        return CmdResultAudioSourceUpdate {
            success: false,
            message: audio_disabled_message(),
        };
    }
    let params = match update_source(
        &mut engine.audio_state,
        args.source_id,
        args.realm_id,
        args.model_id,
        args.position,
        args.velocity,
        args.orientation,
        args.gain,
        args.pitch,
        args.spatial.clone().map(Into::into),
    ) {
        Ok(params) => params,
        Err(message) => {
            return CmdResultAudioSourceUpdate {
                success: false,
                message,
            };
        }
    };
    match engine.audio.source_update(args.source_id, params) {
        Ok(()) => CmdResultAudioSourceUpdate {
            success: true,
            message: "Source updated".into(),
        },
        Err(message) => CmdResultAudioSourceUpdate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_transport(
    engine: &mut EngineState,
    args: &CmdAudioSourceTransportArgs,
) -> CmdResultAudioSourceTransport {
    if let Err(message) = validate_host_logical_id(args.source_id, "sourceId") {
        return CmdResultAudioSourceTransport {
            success: false,
            message,
        };
    }
    if !engine.audio_available {
        return CmdResultAudioSourceTransport {
            success: false,
            message: audio_disabled_message(),
        };
    }
    match args.action {
        AudioSourceTransportActionDto::Play => {
            let Some(resource_id) = args.resource_id else {
                return CmdResultAudioSourceTransport {
                    success: false,
                    message: "resourceId is required for action=play".into(),
                };
            };
            let timeline_id = args.timeline_id.unwrap_or(0);
            if let Err(message) = validate_host_logical_id(resource_id, "resourceId") {
                return CmdResultAudioSourceTransport {
                    success: false,
                    message,
                };
            }
            let mode = args.mode.clone().unwrap_or(AudioPlayModeDto::Once);
            let intensity = args.intensity.unwrap_or(1.0).clamp(0.0, 1.0);
            match engine.audio.source_play(
                args.source_id,
                resource_id,
                timeline_id,
                mode.into(),
                args.delay_ms,
                intensity,
            ) {
                Ok(()) => CmdResultAudioSourceTransport {
                    success: true,
                    message: "Source playing".into(),
                },
                Err(message) => CmdResultAudioSourceTransport {
                    success: false,
                    message,
                },
            }
        }
        AudioSourceTransportActionDto::Pause => {
            match engine.audio.source_pause(args.source_id, args.timeline_id) {
                Ok(()) => CmdResultAudioSourceTransport {
                    success: true,
                    message: "Source paused".into(),
                },
                Err(message) => CmdResultAudioSourceTransport {
                    success: false,
                    message,
                },
            }
        }
        AudioSourceTransportActionDto::Stop => {
            match engine.audio.source_stop(args.source_id, args.timeline_id) {
                Ok(()) => CmdResultAudioSourceTransport {
                    success: true,
                    message: "Source stopped".into(),
                },
                Err(message) => CmdResultAudioSourceTransport {
                    success: false,
                    message,
                },
            }
        }
    }
}

pub fn engine_cmd_audio_source_dispose(
    engine: &mut EngineState,
    args: &CmdAudioSourceDisposeArgs,
) -> CmdResultAudioSourceDispose {
    if let Err(message) = validate_host_logical_id(args.source_id, "sourceId") {
        return CmdResultAudioSourceDispose {
            success: false,
            message,
        };
    }
    if !engine.audio_available {
        return CmdResultAudioSourceDispose {
            success: false,
            message: audio_disabled_message(),
        };
    }
    dispose_source(&mut engine.audio_state, args.source_id);
    match engine.audio.source_dispose(args.source_id) {
        Ok(()) => CmdResultAudioSourceDispose {
            success: true,
            message: "Source disposed".into(),
        },
        Err(message) => CmdResultAudioSourceDispose {
            success: false,
            message,
        },
    }
}

pub fn process_audio_source_bindings(engine: &mut EngineState) {
    if !engine.audio_available {
        return;
    }
    let listener_binding = engine.audio_state.listener_binding;
    let Some(listener_binding) = listener_binding else {
        return;
    };
    let realm_id = crate::core::realm::RealmId(listener_binding.realm_id);
    let entities = match engine.universal_state.scene.realm3d.entities.get(&realm_id) {
        Some(entities) => entities,
        None => return,
    };
    let models: Vec<_> = entities
        .models
        .iter()
        .map(|(&model_id, record)| {
            let (_, rotation, translation) = record.data.transform.to_scale_rotation_translation();
            AudioModelTransform {
                model_id,
                translation,
                rotation,
            }
        })
        .collect();
    for update in plan_bound_source_updates(&engine.audio_state, listener_binding.realm_id, &models)
    {
        let _ = engine.audio.source_update(update.source_id, update.params);
    }
}
