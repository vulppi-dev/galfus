use glam::{Quat, Vec3};

use crate::{AudioListenerState, AudioSourceParams, AudioState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioModelTransform {
    pub model_id: u32,
    pub translation: Vec3,
    pub rotation: Quat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSourceUpdatePlan {
    pub source_id: u32,
    pub params: AudioSourceParams,
}

pub fn resolve_listener_binding_state(
    state: &AudioState,
    realm_id: u32,
    models: &[AudioModelTransform],
) -> Option<AudioListenerState> {
    let binding = state.listener_binding?;
    if binding.realm_id != realm_id {
        return None;
    }
    let model = models
        .iter()
        .find(|model| model.model_id == binding.model_id)?;
    let forward = (model.rotation * Vec3::NEG_Z).normalize_or_zero();
    let up = (model.rotation * Vec3::Y).normalize_or_zero();
    Some(AudioListenerState {
        position: model.translation,
        velocity: Vec3::ZERO,
        forward,
        up,
    })
}

pub fn plan_bound_source_updates(
    state: &AudioState,
    realm_id: u32,
    models: &[AudioModelTransform],
) -> Vec<AudioSourceUpdatePlan> {
    let Some(listener_binding) = state.listener_binding else {
        return Vec::new();
    };
    if listener_binding.realm_id != realm_id {
        return Vec::new();
    }
    let Some(listener_model) = models
        .iter()
        .find(|model| model.model_id == listener_binding.model_id)
    else {
        return Vec::new();
    };

    let mut updates = Vec::new();
    for (&source_id, binding) in &state.source_bindings {
        if binding.realm_id != realm_id {
            continue;
        }
        let Some(model) = models
            .iter()
            .find(|model| model.model_id == binding.model_id)
        else {
            continue;
        };
        let Some(mut params) = state.source_params.get(&source_id).copied() else {
            continue;
        };
        params.position = model.translation;
        params.orientation = model.rotation;
        if binding.model_id == listener_binding.model_id {
            params.position = listener_model.translation;
            params.orientation = listener_model.rotation;
            params.spatial.min_distance = 0.0;
            params.spatial.max_distance = 0.01;
            params.spatial.rolloff = 0.0;
        }
        updates.push(AudioSourceUpdatePlan { source_id, params });
    }
    updates.sort_by_key(|update| update.source_id);
    updates
}

#[cfg(test)]
mod tests {
    use glam::{Quat, Vec3};

    use crate::{AudioSourceParams, AudioState, bind_listener, insert_source};

    use super::{AudioModelTransform, plan_bound_source_updates, resolve_listener_binding_state};

    #[test]
    fn resolve_listener_binding_state_reads_transform_from_bound_model() {
        let mut state = AudioState::default();
        bind_listener(&mut state, 4, 9);
        let listener = resolve_listener_binding_state(
            &state,
            4,
            &[AudioModelTransform {
                model_id: 9,
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::IDENTITY,
            }],
        )
        .expect("listener");
        assert_eq!(listener.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(listener.forward, Vec3::NEG_Z);
        assert_eq!(listener.up, Vec3::Y);
    }

    #[test]
    fn plan_bound_source_updates_tracks_bound_models_and_listener_model() {
        let mut state = AudioState::default();
        bind_listener(&mut state, 7, 10);
        insert_source(&mut state, 2, 7, 10, AudioSourceParams::default());
        insert_source(&mut state, 1, 7, 11, AudioSourceParams::default());

        let updates = plan_bound_source_updates(
            &state,
            7,
            &[
                AudioModelTransform {
                    model_id: 10,
                    translation: Vec3::new(5.0, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                },
                AudioModelTransform {
                    model_id: 11,
                    translation: Vec3::new(9.0, 1.0, 0.0),
                    rotation: Quat::IDENTITY,
                },
            ],
        );

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].source_id, 1);
        assert_eq!(updates[0].params.position, Vec3::new(9.0, 1.0, 0.0));
        assert_eq!(updates[1].source_id, 2);
        assert_eq!(updates[1].params.position, Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(updates[1].params.spatial.min_distance, 0.0);
        assert_eq!(updates[1].params.spatial.max_distance, 0.01);
        assert_eq!(updates[1].params.spatial.rolloff, 0.0);
    }
}
