mod backend;
mod contracts;
mod sync;
#[cfg(not(target_arch = "wasm32"))]
mod backends {
    pub mod kira;
}
#[cfg(target_arch = "wasm32")]
mod backends {
    pub mod webaudio;
}

use glam::{Quat, Vec3};
use std::collections::HashMap;

pub use backend::AudioProxy;
#[cfg(not(target_arch = "wasm32"))]
pub use backends::kira::KiraAudioProxy;
#[cfg(target_arch = "wasm32")]
pub use backends::webaudio::WebAudioProxy;
pub use contracts::*;
pub use sync::{
    AudioModelTransform, AudioSourceUpdatePlan, plan_bound_source_updates,
    resolve_listener_binding_state,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AudioListenerState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSpatialParams {
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: f32,
    pub cone_inner: f32,
    pub cone_outer: f32,
    pub cone_outer_gain: f32,
}

impl Default for AudioSpatialParams {
    fn default() -> Self {
        Self {
            min_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            cone_inner: 360.0,
            cone_outer: 360.0,
            cone_outer_gain: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSourceParams {
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
    pub spatial: AudioSpatialParams,
}

impl Default for AudioSourceParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            gain: 1.0,
            pitch: 1.0,
            spatial: AudioSpatialParams::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlayMode {
    Once,
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioListenerBinding {
    pub realm_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioReadyEvent {
    pub resource_id: u32,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioStreamState {
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub data: Vec<u8>,
    pub ranges: Vec<(u64, u64)>,
}

impl AudioStreamState {
    pub fn new(total_bytes: u64) -> Result<Self, String> {
        let size = usize::try_from(total_bytes)
            .map_err(|_| "Audio stream size exceeds addressable memory".to_string())?;
        Ok(Self {
            total_bytes,
            received_bytes: 0,
            data: vec![0; size],
            ranges: Vec::new(),
        })
    }

    pub fn apply_chunk(&mut self, offset: u64, bytes: &[u8]) -> Result<u64, String> {
        if offset >= self.total_bytes {
            return Err("Chunk offset exceeds total size".into());
        }
        let end = (offset + bytes.len() as u64).min(self.total_bytes);
        let write_len = (end - offset) as usize;
        if write_len == 0 {
            return Ok(0);
        }
        let start_index = offset as usize;
        self.data[start_index..start_index + write_len].copy_from_slice(&bytes[..write_len]);
        let added = Self::merge_range(&mut self.ranges, offset, end);
        self.received_bytes = self.received_bytes.saturating_add(added);
        Ok(added)
    }

    fn merge_range(ranges: &mut Vec<(u64, u64)>, mut start: u64, mut end: u64) -> u64 {
        let mut added = end.saturating_sub(start);
        let mut index = 0;
        while index < ranges.len() {
            let (range_start, range_end) = ranges[index];
            if range_end < start {
                index += 1;
                continue;
            }
            if range_start > end {
                break;
            }
            let overlap_start = start.max(range_start);
            let overlap_end = end.min(range_end);
            if overlap_end > overlap_start {
                added = added.saturating_sub(overlap_end - overlap_start);
            }
            start = start.min(range_start);
            end = end.max(range_end);
            ranges.remove(index);
        }
        ranges.insert(index, (start, end));
        added
    }

    pub fn complete(&self) -> bool {
        self.received_bytes >= self.total_bytes && self.total_bytes > 0
    }
}

#[derive(Debug, Default)]
pub struct AudioState {
    pub listener_binding: Option<AudioListenerBinding>,
    pub source_bindings: HashMap<u32, AudioListenerBinding>,
    pub source_params: HashMap<u32, AudioSourceParams>,
    pub streams: HashMap<u32, AudioStreamState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioListenerBindingSnapshot {
    pub realm_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSourceSnapshot {
    pub source_id: u32,
    pub realm_id: Option<u32>,
    pub model_id: Option<u32>,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioStreamSnapshot {
    pub resource_id: u32,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub complete: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioStateSnapshot {
    pub listener: Option<AudioListenerBindingSnapshot>,
    pub sources: Vec<AudioSourceSnapshot>,
    pub streams: Vec<AudioStreamSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioStreamUpsertResult {
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub complete: bool,
    pub completed_data: Option<Vec<u8>>,
}

pub fn bind_listener(state: &mut AudioState, realm_id: u32, model_id: u32) {
    state.listener_binding = Some(AudioListenerBinding { realm_id, model_id });
}

pub fn dispose_listener_for_realm(state: &mut AudioState, realm_id: u32) -> bool {
    let should_clear = matches!(
        state.listener_binding,
        Some(binding) if binding.realm_id == realm_id
    );
    if should_clear {
        state.listener_binding = None;
    }
    should_clear
}

pub fn insert_source(
    state: &mut AudioState,
    source_id: u32,
    realm_id: u32,
    model_id: u32,
    params: AudioSourceParams,
) {
    state.source_params.insert(source_id, params);
    state
        .source_bindings
        .insert(source_id, AudioListenerBinding { realm_id, model_id });
}

pub fn update_source(
    state: &mut AudioState,
    source_id: u32,
    realm_id: Option<u32>,
    model_id: Option<u32>,
    position: Option<Vec3>,
    velocity: Option<Vec3>,
    orientation: Option<Quat>,
    gain: Option<f32>,
    pitch: Option<f32>,
    spatial: Option<AudioSpatialParams>,
) -> Result<AudioSourceParams, String> {
    let Some(mut params) = state.source_params.get(&source_id).copied() else {
        return Err(format!("Source {} not found", source_id));
    };
    if let Some(position) = position {
        params.position = position;
    }
    if let Some(velocity) = velocity {
        params.velocity = velocity;
    }
    if let Some(orientation) = orientation {
        params.orientation = orientation;
    }
    if let Some(gain) = gain {
        params.gain = gain;
    }
    if let Some(pitch) = pitch {
        params.pitch = pitch;
    }
    if let Some(spatial) = spatial {
        params.spatial = spatial;
    }
    state.source_params.insert(source_id, params);
    if realm_id.is_some() || model_id.is_some() {
        let Some(binding) = state.source_bindings.get_mut(&source_id) else {
            return Err(format!("Source binding {} not found", source_id));
        };
        if let Some(realm_id) = realm_id {
            binding.realm_id = realm_id;
        }
        if let Some(model_id) = model_id {
            binding.model_id = model_id;
        }
    }
    Ok(params)
}

pub fn dispose_source(state: &mut AudioState, source_id: u32) {
    state.source_bindings.remove(&source_id);
    state.source_params.remove(&source_id);
}

pub fn upsert_stream_chunk(
    state: &mut AudioState,
    resource_id: u32,
    total_bytes: Option<u64>,
    offset: u64,
    data: &[u8],
) -> Result<AudioStreamUpsertResult, String> {
    let resolved_total_bytes = total_bytes
        .or_else(|| {
            state
                .streams
                .get(&resource_id)
                .map(|stream| stream.total_bytes)
        })
        .unwrap_or(0);
    let stream = match state.streams.entry(resource_id) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            let stream = AudioStreamState::new(resolved_total_bytes)?;
            entry.insert(stream)
        }
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
    };
    stream.apply_chunk(offset, data)?;
    let complete = stream.complete();
    let received_bytes = stream.received_bytes;
    let total_bytes = stream.total_bytes;
    let completed_data = if complete {
        state.streams.remove(&resource_id).map(|stream| stream.data)
    } else {
        None
    };
    Ok(AudioStreamUpsertResult {
        received_bytes,
        total_bytes,
        complete,
        completed_data,
    })
}

pub fn snapshot_audio_state(
    state: &AudioState,
    include_listener: bool,
    include_sources: bool,
    include_streams: bool,
) -> AudioStateSnapshot {
    let listener = include_listener
        .then(|| {
            state
                .listener_binding
                .map(|binding| AudioListenerBindingSnapshot {
                    realm_id: binding.realm_id,
                    model_id: binding.model_id,
                })
        })
        .flatten();

    let mut sources = if include_sources {
        let mut entries: Vec<_> = state
            .source_params
            .iter()
            .map(|(&source_id, params)| {
                let (realm_id, model_id) = state
                    .source_bindings
                    .get(&source_id)
                    .map(|binding| (Some(binding.realm_id), Some(binding.model_id)))
                    .unwrap_or((None, None));
                AudioSourceSnapshot {
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

    let mut streams = if include_streams {
        let mut entries: Vec<_> = state
            .streams
            .iter()
            .map(|(&resource_id, stream)| AudioStreamSnapshot {
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

    AudioStateSnapshot {
        listener,
        sources: std::mem::take(&mut sources),
        streams: std::mem::take(&mut streams),
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
