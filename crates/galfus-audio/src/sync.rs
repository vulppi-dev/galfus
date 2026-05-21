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
#[path = "sync_tests.rs"]
mod tests;
