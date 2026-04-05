mod bootstrap;
extern crate self as vulfram_runtime;

use std::collections::HashMap;

pub use bootstrap::{
    RenderBootstrapDeviceStrategy, RuntimeRenderBootstrapPlan, plan_render_bootstrap,
};
pub mod core;

pub use core::{
    vulfram_dispose, vulfram_get_profiling, vulfram_init, vulfram_receive_events,
    vulfram_receive_queue, vulfram_send_queue, vulfram_tick, vulfram_upload_buffer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeferredCommandKey {
    pub command_id: u64,
    pub command_signature: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeferredCommandMeta {
    pub first_frame: u64,
    pub attempts: u32,
    pub next_retry_frame: u64,
    pub last_reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredRetryState {
    pub attempts: u32,
    pub age_frames: u64,
    pub next_retry_frame: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeFrameState {
    pub time: u64,
    pub delta_time: u32,
    pub frame_index: u64,
    pub had_commands_this_frame: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState<TCmd, TEvent, TResponse> {
    frame: RuntimeFrameState,
    cmd_queue: Vec<TCmd>,
    deferred_cmd_queue: Vec<TCmd>,
    deferred_cmd_meta: HashMap<DeferredCommandKey, DeferredCommandMeta>,
    event_queue: Vec<TEvent>,
    response_queue: Vec<TResponse>,
}

impl<TCmd, TEvent, TResponse> Default for RuntimeState<TCmd, TEvent, TResponse> {
    fn default() -> Self {
        Self {
            frame: RuntimeFrameState::default(),
            cmd_queue: Vec::new(),
            deferred_cmd_queue: Vec::new(),
            deferred_cmd_meta: HashMap::new(),
            event_queue: Vec::new(),
            response_queue: Vec::new(),
        }
    }
}

impl RuntimeFrameState {
    pub fn begin_tick(&mut self, time: u64, delta_time: u32) {
        self.time = time;
        self.delta_time = delta_time;
        self.had_commands_this_frame = false;
    }

    pub fn advance_frame(&mut self) -> u64 {
        self.frame_index = self.frame_index.wrapping_add(1);
        self.frame_index
    }
}

impl<TCmd, TEvent, TResponse> RuntimeState<TCmd, TEvent, TResponse> {
    pub fn begin_tick(&mut self, time: u64, delta_time: u32) {
        self.frame.begin_tick(time, delta_time);
    }

    pub fn frame_index(&self) -> u64 {
        self.frame.frame_index
    }

    pub fn time_ms(&self) -> u64 {
        self.frame.time
    }

    pub fn delta_time_ms(&self) -> u32 {
        self.frame.delta_time
    }

    pub fn had_commands_this_frame(&self) -> bool {
        self.frame.had_commands_this_frame
    }

    pub fn set_had_commands_this_frame(&mut self, had_commands: bool) {
        self.frame.had_commands_this_frame = had_commands;
    }

    pub fn advance_frame(&mut self) -> u64 {
        self.frame.advance_frame()
    }

    pub fn has_pending_commands(&self) -> bool {
        !self.cmd_queue.is_empty() || !self.deferred_cmd_queue.is_empty()
    }

    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    pub fn clear_responses(&mut self) {
        self.response_queue.clear();
    }

    pub fn enqueue_commands<I>(&mut self, commands: I)
    where
        I: IntoIterator<Item = TCmd>,
    {
        self.cmd_queue.extend(commands);
    }

    pub fn push_deferred_command(&mut self, command: TCmd) {
        self.deferred_cmd_queue.push(command);
    }

    pub fn take_pending_commands(&mut self) -> Vec<TCmd> {
        std::mem::take(&mut self.cmd_queue)
    }

    pub fn take_deferred_commands(&mut self) -> Vec<TCmd> {
        std::mem::take(&mut self.deferred_cmd_queue)
    }

    pub fn replace_deferred_commands(&mut self, commands: Vec<TCmd>) {
        self.deferred_cmd_queue = commands;
    }

    pub fn take_ready_commands<F>(&mut self, frame_index: u64, deferred_key_for: F) -> Vec<TCmd>
    where
        F: Fn(&TCmd) -> DeferredCommandKey,
    {
        let deferred = self.take_deferred_commands();
        let mut batch = self.take_pending_commands();
        let mut still_deferred = Vec::new();

        for command in deferred {
            let key = deferred_key_for(&command);
            if self.deferred_is_ready(&key, frame_index) {
                batch.push(command);
            } else {
                still_deferred.push(command);
            }
        }

        self.replace_deferred_commands(still_deferred);
        batch
    }

    pub fn event_count(&self) -> usize {
        self.event_queue.len()
    }

    pub fn response_count(&self) -> usize {
        self.response_queue.len()
    }

    pub fn events(&self) -> &[TEvent] {
        &self.event_queue
    }

    pub fn event_batch(&self) -> &Vec<TEvent> {
        &self.event_queue
    }

    pub fn responses(&self) -> &[TResponse] {
        &self.response_queue
    }

    pub fn response_batch(&self) -> &Vec<TResponse> {
        &self.response_queue
    }

    pub fn last_response(&self) -> Option<&TResponse> {
        self.response_queue.last()
    }

    pub fn last_response_cloned(&self) -> Option<TResponse>
    where
        TResponse: Clone,
    {
        self.response_queue.last().cloned()
    }

    pub fn pop_response(&mut self) -> Option<TResponse> {
        self.response_queue.pop()
    }

    pub fn take_events(&mut self) -> Vec<TEvent> {
        std::mem::take(&mut self.event_queue)
    }

    pub fn replace_events(&mut self, events: Vec<TEvent>) {
        self.event_queue = events;
    }

    pub fn cloned_events(&self) -> Vec<TEvent>
    where
        TEvent: Clone,
    {
        self.event_queue.clone()
    }

    pub fn push_event(&mut self, event: TEvent) {
        self.event_queue.push(event);
    }

    pub fn push_response(&mut self, response: TResponse) {
        self.response_queue.push(response);
    }

    pub fn deferred_contains(&self, key: &DeferredCommandKey) -> bool {
        self.deferred_cmd_meta.contains_key(key)
    }

    pub fn deferred_is_ready(&self, key: &DeferredCommandKey, frame_index: u64) -> bool {
        self.deferred_cmd_meta
            .get(key)
            .map(|meta| meta.next_retry_frame <= frame_index)
            .unwrap_or(true)
    }

    pub fn record_deferred_retry(
        &mut self,
        key: DeferredCommandKey,
        frame_index: u64,
        reason: &str,
    ) -> DeferredRetryState {
        let meta = self
            .deferred_cmd_meta
            .entry(key)
            .or_insert_with(|| DeferredCommandMeta {
                first_frame: frame_index,
                attempts: 0,
                next_retry_frame: frame_index,
                last_reason: String::new(),
            });
        meta.attempts = meta.attempts.saturating_add(1);
        meta.last_reason = reason.into();
        let backoff = defer_backoff_frames(meta.attempts);
        meta.next_retry_frame = frame_index.saturating_add(backoff);
        DeferredRetryState {
            attempts: meta.attempts,
            age_frames: frame_index.saturating_sub(meta.first_frame),
            next_retry_frame: meta.next_retry_frame,
        }
    }

    pub fn clear_deferred_meta(&mut self, key: &DeferredCommandKey) -> Option<DeferredCommandMeta> {
        self.deferred_cmd_meta.remove(key)
    }
}

pub const DEFER_MAX_ATTEMPTS: u32 = 120;
pub const DEFER_MAX_AGE_FRAMES: u64 = 1200;
pub const DEFER_BACKOFF_MAX_EXP: u32 = 6;

pub fn defer_backoff_frames(attempts: u32) -> u64 {
    let exp = attempts.saturating_sub(1).min(DEFER_BACKOFF_MAX_EXP);
    1_u64 << exp
}

pub fn should_drop_deferred(attempts: u32, age_frames: u64) -> bool {
    attempts >= DEFER_MAX_ATTEMPTS || age_frames >= DEFER_MAX_AGE_FRAMES
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
