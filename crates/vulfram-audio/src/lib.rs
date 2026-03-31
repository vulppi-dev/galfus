use glam::{Quat, Vec3};
use std::collections::HashMap;

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
mod tests {
    use super::{
        AudioListenerBinding, AudioSourceParams, AudioState, AudioStreamState, snapshot_audio_state,
    };
    use glam::Vec3;

    #[test]
    fn audio_stream_merges_overlapping_chunks_without_double_counting() {
        let mut stream = AudioStreamState::new(8).expect("stream");
        assert_eq!(
            stream.apply_chunk(0, &[1, 2, 3, 4]).expect("first chunk"),
            4
        );
        assert_eq!(stream.apply_chunk(2, &[9, 9, 9, 9]).expect("overlap"), 2);
        assert_eq!(stream.received_bytes, 6);
        assert_eq!(stream.ranges, vec![(0, 6)]);
    }

    #[test]
    fn audio_stream_reports_complete_only_after_full_range() {
        let mut stream = AudioStreamState::new(4).expect("stream");
        assert!(!stream.complete());
        let _ = stream.apply_chunk(0, &[1, 2, 3, 4]).expect("full chunk");
        assert!(stream.complete());
    }

    #[test]
    fn snapshot_audio_state_sorts_sources_and_streams() {
        let mut state = AudioState {
            listener_binding: Some(AudioListenerBinding {
                realm_id: 9,
                model_id: 7,
            }),
            ..Default::default()
        };
        state.source_params.insert(
            20,
            AudioSourceParams {
                position: Vec3::new(1.0, 2.0, 3.0),
                ..Default::default()
            },
        );
        state.source_bindings.insert(
            20,
            AudioListenerBinding {
                realm_id: 5,
                model_id: 4,
            },
        );
        state.source_params.insert(10, AudioSourceParams::default());
        let mut stream = AudioStreamState::new(2).expect("stream");
        let _ = stream.apply_chunk(0, &[1, 2]).expect("chunk");
        state.streams.insert(3, stream);

        let snapshot = snapshot_audio_state(&state, true, true, true);

        assert_eq!(snapshot.listener.expect("listener").realm_id, 9);
        assert_eq!(snapshot.sources.len(), 2);
        assert_eq!(snapshot.sources[0].source_id, 10);
        assert_eq!(snapshot.sources[1].source_id, 20);
        assert_eq!(snapshot.sources[1].realm_id, Some(5));
        assert_eq!(snapshot.streams[0].resource_id, 3);
        assert!(snapshot.streams[0].complete);
    }
}
