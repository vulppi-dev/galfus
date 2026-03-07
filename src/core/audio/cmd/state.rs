use crate::core::state::EngineState;

use super::types::{
    AudioListenerBindingState, AudioSourceStateEntry, AudioStreamStateEntry, CmdAudioStateGetArgs,
    CmdResultAudioStateGet, audio_disabled_message,
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

    let listener = if args.include_listener {
        engine
            .audio_state
            .listener_binding
            .map(|binding| AudioListenerBindingState {
                realm_id: binding.realm_id,
                model_id: binding.model_id,
            })
    } else {
        None
    };

    let mut sources = if args.include_sources {
        let mut entries: Vec<_> = engine
            .audio_state
            .source_params
            .iter()
            .map(|(&source_id, params)| {
                let (realm_id, model_id) = engine
                    .audio_state
                    .source_bindings
                    .get(&source_id)
                    .map(|binding| (Some(binding.realm_id), Some(binding.model_id)))
                    .unwrap_or((None, None));
                AudioSourceStateEntry {
                    source_id,
                    realm_id,
                    model_id,
                    position: params.position,
                    velocity: params.velocity,
                    orientation: params.orientation,
                    gain: params.gain,
                    pitch: params.pitch,
                }
            })
            .collect();
        entries.sort_by_key(|entry| entry.source_id);
        entries
    } else {
        Vec::new()
    };

    let mut streams = if args.include_streams {
        let mut entries: Vec<_> = engine
            .audio_state
            .streams
            .iter()
            .map(|(&resource_id, stream)| AudioStreamStateEntry {
                resource_id,
                received_bytes: stream.received_bytes,
                total_bytes: stream.total_bytes,
                complete: stream.complete(),
            })
            .collect();
        entries.sort_by_key(|entry| entry.resource_id);
        entries
    } else {
        Vec::new()
    };

    CmdResultAudioStateGet {
        success: true,
        message: "Audio state listed".into(),
        listener,
        sources: std::mem::take(&mut sources),
        streams: std::mem::take(&mut streams),
    }
}
