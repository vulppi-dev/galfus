use crate::core::target::TargetId;
pub use vulfram_input::{
    CmdInputTargetListenerDisposeArgs, CmdInputTargetListenerListArgs,
    CmdInputTargetListenerUpsertArgs, CmdResultInputTargetListenerList,
    InputTargetListenerSnapshot,
};

#[derive(Debug, Clone)]
pub struct InputTargetListenerConfig {
    pub listener_id: u64,
    pub target_id: TargetId,
    pub enabled: bool,
    pub events: Vec<String>,
    pub sample_percent: u8,
}
