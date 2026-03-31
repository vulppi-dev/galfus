pub mod cmd;
pub mod proxy;

#[cfg(not(feature = "wasm"))]
pub mod kira;
#[cfg(feature = "wasm")]
pub mod webaudio;

pub use cmd::*;
#[cfg(not(feature = "wasm"))]
pub use kira::KiraAudioProxy;
pub use proxy::*;
pub use vulfram_audio::{
    AudioListenerState, AudioPlayMode, AudioReadyEvent, AudioSourceParams, AudioSpatialParams,
};
#[cfg(feature = "wasm")]
pub use webaudio::WebAudioProxy;
