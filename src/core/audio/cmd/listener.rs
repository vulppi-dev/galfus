use glam::Vec3;

use crate::core::audio::AudioListenerState;
use crate::core::state::EngineState;

use super::types::{
    CmdAudioListenerCreateArgs, CmdAudioListenerDisposeArgs, CmdAudioListenerUpdateArgs,
    CmdResultAudioListenerCreate, CmdResultAudioListenerDispose, CmdResultAudioListenerUpdate,
    audio_disabled_message,
};

pub fn engine_cmd_audio_listener_update(
    engine: &mut EngineState,
    args: &CmdAudioListenerUpdateArgs,
) -> CmdResultAudioListenerUpdate {
    if !engine.audio_available {
        return CmdResultAudioListenerUpdate {
            success: false,
            message: audio_disabled_message(),
        };
    }
    let state = AudioListenerState {
        position: args.position,
        velocity: args.velocity,
        forward: args.forward,
        up: args.up,
    };
    match engine.audio.listener_update(state) {
        Ok(()) => CmdResultAudioListenerUpdate {
            success: true,
            message: "Listener updated".into(),
        },
        Err(message) => CmdResultAudioListenerUpdate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_listener_create(
    engine: &mut EngineState,
    args: &CmdAudioListenerCreateArgs,
) -> CmdResultAudioListenerCreate {
    if !engine.audio_available {
        return CmdResultAudioListenerCreate {
            success: false,
            message: audio_disabled_message(),
        };
    }
    engine.audio_state.listener_binding = Some(crate::core::audio::AudioListenerBinding {
        realm_id: args.realm_id,
        model_id: args.model_id,
    });
    CmdResultAudioListenerCreate {
        success: true,
        message: "Listener bound to model".into(),
    }
}

pub fn engine_cmd_audio_listener_dispose(
    engine: &mut EngineState,
    args: &CmdAudioListenerDisposeArgs,
) -> CmdResultAudioListenerDispose {
    if !engine.audio_available {
        return CmdResultAudioListenerDispose {
            success: false,
            message: audio_disabled_message(),
        };
    }
    let should_clear = match engine.audio_state.listener_binding {
        Some(binding) => binding.realm_id == args.realm_id,
        None => false,
    };
    if should_clear {
        engine.audio_state.listener_binding = None;
        CmdResultAudioListenerDispose {
            success: true,
            message: "Listener disposed".into(),
        }
    } else {
        CmdResultAudioListenerDispose {
            success: false,
            message: "Listener not found".into(),
        }
    }
}

pub fn process_audio_listener_binding(engine: &mut EngineState) {
    if !engine.audio_available {
        return;
    }
    let binding = match engine.audio_state.listener_binding {
        Some(binding) => binding,
        None => return,
    };
    let realm_id = crate::core::realm::RealmId(binding.realm_id);
    let entities = match engine.universal_state.realm_entities.get(&realm_id) {
        Some(entities) => entities,
        None => return,
    };
    let record = match entities.models.get(&binding.model_id) {
        Some(record) => record,
        None => return,
    };
    let (_, rotation, translation) = record.data.transform.to_scale_rotation_translation();
    let forward = (rotation * Vec3::NEG_Z).normalize_or_zero();
    let up = (rotation * Vec3::Y).normalize_or_zero();
    let state = AudioListenerState {
        position: translation,
        velocity: Vec3::ZERO,
        forward,
        up,
    };
    let _ = engine.audio.listener_update(state);
}
