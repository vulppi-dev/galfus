use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeFrameState {
    pub time: u64,
    pub delta_time: u32,
    pub frame_index: u64,
    pub had_commands_this_frame: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState<TCmd, TEvent, TResponse> {
    pub frame: RuntimeFrameState,
    pub cmd_queue: Vec<TCmd>,
    pub deferred_cmd_queue: Vec<TCmd>,
    pub deferred_cmd_meta: HashMap<DeferredCommandKey, DeferredCommandMeta>,
    pub event_queue: Vec<TEvent>,
    pub response_queue: Vec<TResponse>,
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
mod tests {
    use super::{
        DEFER_MAX_AGE_FRAMES, DEFER_MAX_ATTEMPTS, RuntimeFrameState, RuntimeState,
        defer_backoff_frames, should_drop_deferred,
    };

    #[test]
    fn defer_backoff_caps_at_sixty_four_frames() {
        assert_eq!(defer_backoff_frames(1), 1);
        assert_eq!(defer_backoff_frames(2), 2);
        assert_eq!(defer_backoff_frames(3), 4);
        assert_eq!(defer_backoff_frames(7), 64);
        assert_eq!(defer_backoff_frames(100), 64);
    }

    #[test]
    fn deferred_drop_policy_uses_attempts_or_age() {
        assert!(should_drop_deferred(DEFER_MAX_ATTEMPTS, 0));
        assert!(should_drop_deferred(0, DEFER_MAX_AGE_FRAMES));
        assert!(!should_drop_deferred(
            DEFER_MAX_ATTEMPTS - 1,
            DEFER_MAX_AGE_FRAMES - 1
        ));
    }

    #[test]
    fn runtime_frame_state_tracks_tick_and_wraps_frame_index() {
        let mut state = RuntimeFrameState::default();
        state.begin_tick(100, 16);
        assert_eq!(state.time, 100);
        assert_eq!(state.delta_time, 16);
        assert!(!state.had_commands_this_frame);
        assert_eq!(state.advance_frame(), 1);
    }

    #[test]
    fn runtime_state_starts_with_empty_queues() {
        let state = RuntimeState::<u8, u16, u32>::default();
        assert!(state.cmd_queue.is_empty());
        assert!(state.deferred_cmd_queue.is_empty());
        assert!(state.deferred_cmd_meta.is_empty());
        assert!(state.event_queue.is_empty());
        assert!(state.response_queue.is_empty());
    }
}
