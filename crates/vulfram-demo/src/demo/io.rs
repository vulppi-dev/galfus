use rmp_serde::{from_slice, to_vec_named};
use std::sync::atomic::{AtomicU64, Ordering};
use vulfram_core::core::VulframResult;
use vulfram_core::core::cmd::{CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope, EngineEvent};

use vulfram_core::core;

static NEXT_COMMAND_ID: AtomicU64 = AtomicU64::new(1);

pub fn send_commands(cmds: Vec<EngineCmd>) -> VulframResult {
    let envelopes: Vec<EngineCmdEnvelope> = cmds
        .into_iter()
        .map(|cmd| EngineCmdEnvelope {
            id: NEXT_COMMAND_ID.fetch_add(1, Ordering::Relaxed),
            cmd,
        })
        .collect();
    let data = to_vec_named(&envelopes).expect("failed to serialize commands");
    core::vulfram_send_queue(data.as_ptr(), data.len())
}

pub fn receive_responses() -> Vec<CommandResponseEnvelope> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_queue(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    from_slice(&bytes).expect("failed to deserialize responses")
}

pub fn receive_events() -> Vec<EngineEvent> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_events(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    from_slice(&bytes).expect("failed to deserialize events")
}
