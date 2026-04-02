use crate::core::state::EngineState;
use vulfram_audio::snapshot_audio_state;

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
