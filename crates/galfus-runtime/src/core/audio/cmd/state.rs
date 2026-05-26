use crate::core::id_policy::validate_host_logical_id;
use crate::core::state::EngineState;
use galfus_audio::snapshot_audio_state;

use super::types::{
    AudioListenerBindingState, AudioSourceStateEntry, AudioStreamStateEntry,
    CmdAudioListenerGetArgs, CmdAudioResourceGetArgs, CmdAudioResourceListArgs,
    CmdAudioSourceGetArgs, CmdAudioSourceListArgs, CmdAudioStateGetArgs, CmdResultAudioListenerGet,
    CmdResultAudioResourceGet, CmdResultAudioResourceList, CmdResultAudioSourceGet,
    CmdResultAudioSourceList, CmdResultAudioStateGet, audio_disabled_message,
};

pub fn engine_cmd_audio_state_get(
    engine: &mut EngineState,
    args: &CmdAudioStateGetArgs,
) -> CmdResultAudioStateGet {
    if !engine.audio_available {
        return CmdResultAudioStateGet {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }

    let snapshot = snapshot_audio_state(
        &engine.audio_state,
        args.include_listener,
        args.include_sources,
        args.include_streams,
    );

    CmdResultAudioStateGet {
        success: true,
        message: "Audio state listed".into(),
        listener: snapshot.listener.map(|binding| AudioListenerBindingState {
            realm_id: binding.realm_id,
            model_id: binding.model_id,
        }),
        sources: snapshot
            .sources
            .into_iter()
            .map(|entry| AudioSourceStateEntry {
                source_id: entry.source_id,
                realm_id: entry.realm_id,
                model_id: entry.model_id,
                position: entry.position,
                velocity: entry.velocity,
                orientation: entry.orientation,
                gain: entry.gain,
                pitch: entry.pitch,
            })
            .collect(),
        streams: snapshot
            .streams
            .into_iter()
            .map(|entry| AudioStreamStateEntry {
                resource_id: entry.resource_id,
                received_bytes: entry.received_bytes,
                total_bytes: entry.total_bytes,
                complete: entry.complete,
            })
            .collect(),
    }
}

pub fn engine_cmd_audio_listener_get(
    engine: &mut EngineState,
    _args: &CmdAudioListenerGetArgs,
) -> CmdResultAudioListenerGet {
    if !engine.audio_available {
        return CmdResultAudioListenerGet {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }
    let listener = engine
        .audio_state
        .listener_binding
        .map(|binding| AudioListenerBindingState {
            realm_id: binding.realm_id,
            model_id: binding.model_id,
        });
    CmdResultAudioListenerGet {
        success: listener.is_some(),
        message: if listener.is_some() {
            "Audio listener found".into()
        } else {
            "Audio listener not found".into()
        },
        listener,
    }
}

pub fn engine_cmd_audio_source_get(
    engine: &mut EngineState,
    args: &CmdAudioSourceGetArgs,
) -> CmdResultAudioSourceGet {
    if let Err(message) = validate_host_logical_id(args.source_id, "sourceId") {
        return CmdResultAudioSourceGet {
            success: false,
            message,
            ..Default::default()
        };
    }
    if !engine.audio_available {
        return CmdResultAudioSourceGet {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }
    let Some(params) = engine.audio_state.source_params.get(&args.source_id) else {
        return CmdResultAudioSourceGet {
            success: false,
            message: format!("Audio source {} not found", args.source_id),
            ..Default::default()
        };
    };
    let binding = engine.audio_state.source_bindings.get(&args.source_id);
    let source = AudioSourceStateEntry {
        source_id: args.source_id,
        realm_id: binding.map(|b| b.realm_id),
        model_id: binding.map(|b| b.model_id),
        position: params.position,
        velocity: params.velocity,
        orientation: params.orientation,
        gain: params.gain,
        pitch: params.pitch,
    };
    CmdResultAudioSourceGet {
        success: true,
        message: "Audio source found".into(),
        source: Some(source),
    }
}

pub fn engine_cmd_audio_resource_get(
    engine: &mut EngineState,
    args: &CmdAudioResourceGetArgs,
) -> CmdResultAudioResourceGet {
    if let Err(message) = validate_host_logical_id(args.resource_id, "resourceId") {
        return CmdResultAudioResourceGet {
            success: false,
            message,
            ..Default::default()
        };
    }
    if !engine.audio_available {
        return CmdResultAudioResourceGet {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }
    let Some(stream) = engine.audio_state.streams.get(&args.resource_id) else {
        return CmdResultAudioResourceGet {
            success: false,
            message: format!("Audio resource {} not found", args.resource_id),
            ..Default::default()
        };
    };
    CmdResultAudioResourceGet {
        success: true,
        message: "Audio resource found".into(),
        stream: Some(AudioStreamStateEntry {
            resource_id: args.resource_id,
            received_bytes: stream.received_bytes,
            total_bytes: stream.total_bytes,
            complete: stream.complete(),
        }),
    }
}

pub fn engine_cmd_audio_source_list(
    engine: &mut EngineState,
    args: &CmdAudioSourceListArgs,
) -> CmdResultAudioSourceList {
    if let Some(source_ids) = args.source_ids.as_ref() {
        for source_id in source_ids {
            if let Err(message) = validate_host_logical_id(*source_id, "sourceId") {
                return CmdResultAudioSourceList {
                    success: false,
                    message,
                    ..Default::default()
                };
            }
        }
    }
    if !engine.audio_available {
        return CmdResultAudioSourceList {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }
    let sources = engine
        .audio_state
        .source_params
        .iter()
        .filter(|(source_id, _)| {
            args.source_ids
                .as_ref()
                .is_none_or(|ids| ids.contains(source_id))
        })
        .map(|(&source_id, params)| {
            let binding = engine.audio_state.source_bindings.get(&source_id);
            AudioSourceStateEntry {
                source_id,
                realm_id: binding.map(|b| b.realm_id),
                model_id: binding.map(|b| b.model_id),
                position: params.position,
                velocity: params.velocity,
                orientation: params.orientation,
                gain: params.gain,
                pitch: params.pitch,
            }
        })
        .collect();
    CmdResultAudioSourceList {
        success: true,
        message: "Audio sources listed".into(),
        sources,
    }
}

pub fn engine_cmd_audio_resource_list(
    engine: &mut EngineState,
    args: &CmdAudioResourceListArgs,
) -> CmdResultAudioResourceList {
    if let Some(resource_ids) = args.resource_ids.as_ref() {
        for resource_id in resource_ids {
            if let Err(message) = validate_host_logical_id(*resource_id, "resourceId") {
                return CmdResultAudioResourceList {
                    success: false,
                    message,
                    ..Default::default()
                };
            }
        }
    }
    if !engine.audio_available {
        return CmdResultAudioResourceList {
            success: false,
            message: audio_disabled_message(),
            ..Default::default()
        };
    }
    let streams = engine
        .audio_state
        .streams
        .iter()
        .filter(|(resource_id, _)| {
            args.resource_ids
                .as_ref()
                .is_none_or(|ids| ids.contains(resource_id))
        })
        .map(|(&resource_id, stream)| AudioStreamStateEntry {
            resource_id,
            received_bytes: stream.received_bytes,
            total_bytes: stream.total_bytes,
            complete: stream.complete(),
        })
        .collect();
    CmdResultAudioResourceList {
        success: true,
        message: "Audio resources listed".into(),
        streams,
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
