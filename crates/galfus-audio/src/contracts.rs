use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::{AudioPlayMode, AudioSpatialParams};

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
    #[serde(default = "default_vec3_zero")]
    pub position: Vec3,
    #[serde(default = "default_vec3_zero")]
    pub velocity: Vec3,
    #[serde(default = "default_quat_identity")]
    pub orientation: Quat,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default = "default_pitch")]
    pub pitch: f32,
    #[serde(default)]
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

fn default_vec3_zero() -> Vec3 {
    Vec3::ZERO
}

fn default_quat_identity() -> Quat {
    Quat::IDENTITY
}

fn default_gain() -> f32 {
    1.0
}

fn default_pitch() -> f32 {
    1.0
}

impl From<AudioSpatialParamsDto> for AudioSpatialParams {
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

impl From<AudioPlayModeDto> for AudioPlayMode {
    fn from(value: AudioPlayModeDto) -> Self {
        match value {
            AudioPlayModeDto::Once => Self::Once,
            AudioPlayModeDto::Loop => Self::Loop,
        }
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

pub fn audio_disabled_message() -> String {
    "Audio backend unavailable".into()
}
