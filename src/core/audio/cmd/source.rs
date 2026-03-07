use crate::core::audio::AudioSourceParams;
use crate::core::state::EngineState;

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

    engine
        .audio_state
        .source_params
        .insert(args.source_id, params);
    engine.audio_state.source_bindings.insert(
        args.source_id,
        crate::core::audio::AudioListenerBinding {
            realm_id: args.realm_id,
            model_id: args.model_id,
        },
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
    if !engine.audio_available {
        return CmdResultAudioSourceUpdate {
            success: false,
            message: audio_disabled_message(),
        };
    }
    let mut params = match engine
        .audio_state
        .source_params
        .get(&args.source_id)
        .copied()
    {
        Some(params) => params,
        None => {
            return CmdResultAudioSourceUpdate {
                success: false,
                message: format!("Source {} not found", args.source_id),
            };
        }
    };
    if let Some(position) = args.position {
        params.position = position;
    }
    if let Some(velocity) = args.velocity {
        params.velocity = velocity;
    }
    if let Some(orientation) = args.orientation {
        params.orientation = orientation;
    }
    if let Some(gain) = args.gain {
        params.gain = gain;
    }
    if let Some(pitch) = args.pitch {
        params.pitch = pitch;
    }
    if let Some(spatial) = args.spatial.clone() {
        params.spatial = spatial.into();
    }
    engine
        .audio_state
        .source_params
        .insert(args.source_id, params);
    if args.realm_id.is_some() || args.model_id.is_some() {
        let Some(binding) = engine.audio_state.source_bindings.get_mut(&args.source_id) else {
            return CmdResultAudioSourceUpdate {
                success: false,
                message: format!("Source binding {} not found", args.source_id),
            };
        };
        if let Some(realm_id) = args.realm_id {
            binding.realm_id = realm_id;
        }
        if let Some(model_id) = args.model_id {
            binding.model_id = model_id;
        }
    }
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
    if !engine.audio_available {
        return CmdResultAudioSourceDispose {
            success: false,
            message: audio_disabled_message(),
        };
    }
    engine.audio_state.source_bindings.remove(&args.source_id);
    engine.audio_state.source_params.remove(&args.source_id);
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
    let entities = match engine.universal_state.realm_entities.get(&realm_id) {
        Some(entities) => entities,
        None => return,
    };
    let listener_record = match entities.models.get(&listener_binding.model_id) {
        Some(record) => record,
        None => return,
    };
    let (_, listener_rotation, listener_translation) = listener_record
        .data
        .transform
        .to_scale_rotation_translation();
    for (source_id, binding) in engine.audio_state.source_bindings.iter() {
        if binding.realm_id != listener_binding.realm_id {
            continue;
        }
        let record = match entities.models.get(&binding.model_id) {
            Some(record) => record,
            None => continue,
        };
        let (_, rotation, translation) = record.data.transform.to_scale_rotation_translation();
        let mut params = match engine.audio_state.source_params.get(source_id) {
            Some(params) => *params,
            None => continue,
        };
        params.position = translation;
        params.orientation = rotation;
        if binding.model_id == listener_binding.model_id {
            params.position = listener_translation;
            params.orientation = listener_rotation;
            params.spatial.min_distance = 0.0;
            params.spatial.max_distance = 0.01;
            params.spatial.rolloff = 0.0;
        }
        let _ = engine.audio.source_update(*source_id, params);
    }
}
