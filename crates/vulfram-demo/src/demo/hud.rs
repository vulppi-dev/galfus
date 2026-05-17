use vulfram_core::core::cmd::EngineCmd;

pub struct FpsHud;

impl FpsHud {
    pub fn new(_demo_number: u32) -> Self {
        Self
    }

    pub fn setup_commands(&self, _realm_id: u32) -> Vec<EngineCmd> {
        Vec::new()
    }

    pub fn frame_commands(&mut self, _total_ms: u64, _delta_ms: u32) -> Vec<EngineCmd> {
        Vec::new()
    }
}
