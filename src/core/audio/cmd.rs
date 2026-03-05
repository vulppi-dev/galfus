use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::core::audio::{AudioListenerState, AudioSourceParams};
use crate::core::buffers::state::UploadType;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerUpdateArgs {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerCreateArgs {
    pub realm_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerDisposeArgs {
    pub realm_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioResourceUpsertArgs {
    pub resource_id: u32,
    pub buffer_id: u64,
    #[serde(default)]
    pub total_bytes: Option<u64>,
    #[serde(default)]
    pub offset_bytes: Option<u64>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceUpsert {
    pub success: bool,
    pub message: String,
    pub pending: bool,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub complete: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceCreateArgs {
    pub realm_id: u32,
    pub source_id: u32,
    pub model_id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
    pub spatial: AudioSpatialParamsDto,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioSpatialParamsDto {
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: f32,
    pub cone_inner: f32,
    pub cone_outer: f32,
    pub cone_outer_gain: f32,
}

impl Default for AudioSpatialParamsDto {
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

impl From<AudioSpatialParamsDto> for crate::core::audio::AudioSpatialParams {
    fn from(value: AudioSpatialParamsDto) -> Self {
        Self {
            min_distance: value.min_distance,
            max_distance: value.max_distance,
            rolloff: value.rolloff,
            cone_inner: value.cone_inner,
            cone_outer: value.cone_outer,
            cone_outer_gain: value.cone_outer_gain,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceUpdateArgs {
    pub source_id: u32,
    pub realm_id: Option<u32>,
    pub model_id: Option<u32>,
    pub position: Option<Vec3>,
    pub velocity: Option<Vec3>,
    pub orientation: Option<Quat>,
    pub gain: Option<f32>,
    pub pitch: Option<f32>,
    pub spatial: Option<AudioSpatialParamsDto>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultAudioSourceUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum AudioSourceTransportActionDto {
    Play,
    Pause,
    Stop,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceTransportArgs {
    pub source_id: u32,
    pub action: AudioSourceTransportActionDto,
    #[serde(default)]
    pub resource_id: Option<u32>,
    #[serde(default)]
    pub timeline_id: Option<u32>,
    #[serde(default)]
    pub intensity: Option<f32>,
    #[serde(default)]
    pub delay_ms: Option<u32>,
    #[serde(default)]
    pub mode: Option<AudioPlayModeDto>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceTransport {
    pub success: bool,
    pub message: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceDisposeArgs {
    pub source_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum AudioPlayModeDto {
    Once,
    Loop,
}

impl From<AudioPlayModeDto> for crate::core::audio::AudioPlayMode {
    fn from(value: AudioPlayModeDto) -> Self {
        match value {
            AudioPlayModeDto::Once => Self::Once,
            AudioPlayModeDto::Loop => Self::Loop,
        }
    }
}

#[derive(Debug, Clone)]
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
        let mut i = 0;
        while i < ranges.len() {
            let (s, e) = ranges[i];
            if e < start {
                i += 1;
                continue;
            }
            if s > end {
                break;
            }
            let overlap_start = start.max(s);
            let overlap_end = end.min(e);
            if overlap_end > overlap_start {
                added = added.saturating_sub(overlap_end - overlap_start);
            }
            start = start.min(s);
            end = end.max(e);
            ranges.remove(i);
        }
        ranges.insert(i, (start, end));
        added
    }

    pub fn complete(&self) -> bool {
        self.received_bytes >= self.total_bytes && self.total_bytes > 0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioResourceDisposeArgs {
    pub resource_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioStateGetArgs {
    #[serde(default = "default_true")]
    pub include_listener: bool,
    #[serde(default = "default_true")]
    pub include_sources: bool,
    #[serde(default = "default_true")]
    pub include_streams: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioListenerBindingState {
    pub realm_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceStateEntry {
    pub source_id: u32,
    pub realm_id: Option<u32>,
    pub model_id: Option<u32>,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamStateEntry {
    pub resource_id: u32,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub complete: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioStateGet {
    pub success: bool,
    pub message: String,
    pub listener: Option<AudioListenerBindingState>,
    pub sources: Vec<AudioSourceStateEntry>,
    pub streams: Vec<AudioStreamStateEntry>,
}

fn default_true() -> bool {
    true
}

fn audio_disabled_message() -> String {
    "Audio backend unavailable".into()
}

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

pub fn engine_cmd_audio_resource_upsert(
    engine: &mut EngineState,
    args: &CmdAudioResourceUpsertArgs,
) -> CmdResultAudioResourceUpsert {
    if !engine.audio_available {
        return CmdResultAudioResourceUpsert {
            success: false,
            message: audio_disabled_message(),
            pending: false,
            received_bytes: 0,
            total_bytes: 0,
            complete: false,
        };
    }
    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultAudioResourceUpsert {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                pending: false,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            };
        }
    };

    if buffer.upload_type != UploadType::BinaryAsset {
        return CmdResultAudioResourceUpsert {
            success: false,
            message: format!(
                "Invalid buffer type. Expected BinaryAsset, got {:?}",
                buffer.upload_type
            ),
            pending: false,
            received_bytes: 0,
            total_bytes: 0,
            complete: false,
        };
    }

    let offset = args.offset_bytes.unwrap_or(0);
    let has_stream = engine.audio_state.streams.contains_key(&args.resource_id);
    let is_stream_upsert = args.total_bytes.is_some() || has_stream;

    if is_stream_upsert {
        let total_bytes = if let Some(total_bytes) = args.total_bytes {
            total_bytes
        } else {
            match engine.audio_state.streams.get(&args.resource_id) {
                Some(stream) => stream.total_bytes,
                None => 0,
            }
        };
        let stream = match engine.audio_state.streams.entry(args.resource_id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                match AudioStreamState::new(total_bytes) {
                    Ok(state) => entry.insert(state),
                    Err(message) => {
                        return CmdResultAudioResourceUpsert {
                            success: false,
                            message,
                            pending: false,
                            received_bytes: 0,
                            total_bytes: 0,
                            complete: false,
                        };
                    }
                }
            }
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        };
        if let Err(message) = stream.apply_chunk(offset, &buffer.data) {
            return CmdResultAudioResourceUpsert {
                success: false,
                message,
                pending: false,
                received_bytes: stream.received_bytes,
                total_bytes: stream.total_bytes,
                complete: stream.complete(),
            };
        }
        let complete = stream.complete();
        engine
            .event_queue
            .push(crate::core::cmd::EngineEvent::System(
                crate::core::system::events::SystemEvent::AudioStreamProgress {
                    resource_id: args.resource_id,
                    received_bytes: stream.received_bytes,
                    total_bytes: stream.total_bytes,
                    complete,
                },
            ));
        if complete {
            let Some(stream) = engine.audio_state.streams.remove(&args.resource_id) else {
                return CmdResultAudioResourceUpsert {
                    success: false,
                    message: format!("Audio stream {} not found", args.resource_id),
                    pending: false,
                    received_bytes: 0,
                    total_bytes: 0,
                    complete: false,
                };
            };
            match engine
                .audio
                .buffer_create_from_bytes(args.resource_id, stream.data)
            {
                Ok(()) => CmdResultAudioResourceUpsert {
                    success: true,
                    message: "Audio stream queued".into(),
                    pending: true,
                    received_bytes: stream.received_bytes,
                    total_bytes: stream.total_bytes,
                    complete: true,
                },
                Err(message) => CmdResultAudioResourceUpsert {
                    success: false,
                    message,
                    pending: false,
                    received_bytes: stream.received_bytes,
                    total_bytes: stream.total_bytes,
                    complete: true,
                },
            }
        } else {
            CmdResultAudioResourceUpsert {
                success: true,
                message: "Audio stream chunk queued".into(),
                pending: true,
                received_bytes: stream.received_bytes,
                total_bytes: stream.total_bytes,
                complete: false,
            }
        }
    } else {
        match engine
            .audio
            .buffer_create_from_bytes(args.resource_id, buffer.data)
        {
            Ok(()) => CmdResultAudioResourceUpsert {
                success: true,
                message: "Audio buffer queued".into(),
                pending: true,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            },
            Err(message) => CmdResultAudioResourceUpsert {
                success: false,
                message,
                pending: false,
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            },
        }
    }
}

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

pub fn engine_cmd_audio_resource_dispose(
    engine: &mut EngineState,
    args: &CmdAudioResourceDisposeArgs,
) -> CmdResultAudioResourceDispose {
    if !engine.audio_available {
        return CmdResultAudioResourceDispose {
            success: false,
            message: audio_disabled_message(),
        };
    }
    engine.audio_state.streams.remove(&args.resource_id);
    match engine.audio.buffer_dispose(args.resource_id) {
        Ok(()) => CmdResultAudioResourceDispose {
            success: true,
            message: "Resource disposed".into(),
        },
        Err(message) => CmdResultAudioResourceDispose {
            success: false,
            message,
        },
    }
}

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
