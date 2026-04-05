use crate::core::audio::AudioListenerState;
use crate::core::state::EngineState;
use vulfram_audio::{
    AudioModelTransform, bind_listener, dispose_listener_for_realm, resolve_listener_binding_state,
};

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
    bind_listener(&mut engine.audio_state, args.realm_id, args.model_id);
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
    let should_clear = dispose_listener_for_realm(&mut engine.audio_state, args.realm_id);
    if should_clear {
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
    let Some(state) =
        resolve_listener_binding_state(&engine.audio_state, binding.realm_id, &models)
    else {
        return;
    };
    let _ = engine.audio.listener_update(state);
}
