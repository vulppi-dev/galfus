#[allow(unused_imports)]
pub use galfus_audio::{
    AudioListenerBindingState, AudioPlayModeDto, AudioSourceStateEntry,
    AudioSourceTransportActionDto, AudioSpatialParamsDto, AudioStreamStateEntry,
    CmdAudioListenerCreateArgs, CmdAudioListenerDisposeArgs, CmdAudioListenerUpdateArgs,
    CmdAudioResourceDisposeArgs, CmdAudioResourceUpsertArgs, CmdAudioSourceCreateArgs,
    CmdAudioSourceDisposeArgs, CmdAudioSourceTransportArgs, CmdAudioSourceUpdateArgs,
    CmdAudioStateGetArgs, CmdResultAudioListenerCreate, CmdResultAudioListenerDispose,
    CmdResultAudioListenerUpdate, CmdResultAudioResourceDispose, CmdResultAudioResourceUpsert,
    CmdResultAudioSourceCreate, CmdResultAudioSourceDispose, CmdResultAudioSourceTransport,
    CmdResultAudioSourceUpdate, CmdResultAudioStateGet, audio_disabled_message,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioListenerGetArgs {}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerGet {
    pub success: bool,
    pub message: String,
    pub listener: Option<AudioListenerBindingState>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioSourceGetArgs {
    pub source_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceGet {
    pub success: bool,
    pub message: String,
    pub source: Option<AudioSourceStateEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioResourceGetArgs {
    pub resource_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceGet {
    pub success: bool,
    pub message: String,
    pub stream: Option<AudioStreamStateEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioSourceListArgs {
    pub source_ids: Option<Vec<u32>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceList {
    pub success: bool,
    pub message: String,
    pub sources: Vec<AudioSourceStateEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdAudioResourceListArgs {
    pub resource_ids: Option<Vec<u32>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceList {
    pub success: bool,
    pub message: String,
    pub streams: Vec<AudioStreamStateEntry>,
}
