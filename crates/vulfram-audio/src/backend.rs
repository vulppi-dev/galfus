use crate::{AudioListenerState, AudioPlayMode, AudioReadyEvent, AudioSourceParams};

pub trait AudioProxy: Send {
    fn init(&mut self) -> Result<(), String>;
    fn listener_update(&mut self, state: AudioListenerState) -> Result<(), String>;

    fn buffer_create_from_bytes(&mut self, resource_id: u32, bytes: Vec<u8>) -> Result<(), String>;

    fn source_create(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String>;
    fn source_update(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String>;
    fn source_play(
        &mut self,
        source_id: u32,
        resource_id: u32,
        timeline_id: u32,
        mode: AudioPlayMode,
        delay_ms: Option<u32>,
        intensity: f32,
    ) -> Result<(), String>;
    fn source_pause(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String>;
    fn source_stop(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String>;

    fn buffer_dispose(&mut self, resource_id: u32) -> Result<(), String>;
    fn source_dispose(&mut self, source_id: u32) -> Result<(), String>;

    fn drain_events(&mut self) -> Vec<AudioReadyEvent>;
}
