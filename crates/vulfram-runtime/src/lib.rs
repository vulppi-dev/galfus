mod bootstrap;

use std::collections::HashMap;

pub use bootstrap::{
    RenderBootstrapDeviceStrategy, RuntimeRenderBootstrapPlan, plan_render_bootstrap,
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

impl<TCmd, TEvent, TResponse> RuntimeState<TCmd, TEvent, TResponse> {
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

    pub fn event_queue_ref(&self) -> &Vec<TEvent> {
        &self.event_queue
    }

    pub fn events(&self) -> &[TEvent] {
        &self.event_queue
    }

    pub fn response_queue_ref(&self) -> &Vec<TResponse> {
        &self.response_queue
    }

    pub fn responses(&self) -> &[TResponse] {
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
mod tests {
    use super::{
        DEFER_MAX_AGE_FRAMES, DEFER_MAX_ATTEMPTS, DeferredCommandKey,
        RenderBootstrapDeviceStrategy, RuntimeFrameState, RuntimeRenderBootstrapPlan, RuntimeState,
        defer_backoff_frames, plan_render_bootstrap, should_drop_deferred,
    };
    use vulfram_platform::{
        PlatformRenderBootstrapTarget, PlatformRenderSurfaceKind, PlatformSurfaceAlphaMode,
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

    #[test]
    fn runtime_state_queue_helpers_move_batches_without_leaks() {
        let mut state = RuntimeState::<u8, u16, u32>::default();
        state.enqueue_commands([1, 2, 3]);
        state.push_deferred_command(9);
        state.push_event(7);
        state.push_response(11);

        assert!(state.has_pending_commands());
        assert_eq!(state.event_count(), 1);
        assert_eq!(state.response_count(), 1);
        assert_eq!(state.events(), &[7]);
        assert_eq!(state.responses(), &[11]);
        assert_eq!(state.take_pending_commands(), vec![1, 2, 3]);
        assert_eq!(state.take_deferred_commands(), vec![9]);
        assert_eq!(state.event_queue_ref(), &vec![7]);
        assert_eq!(state.response_queue_ref(), &vec![11]);
        assert_eq!(state.take_events(), vec![7]);
        state.replace_deferred_commands(vec![5, 6]);
        state.replace_events(vec![8]);
        state.clear_events();
        state.clear_responses();

        assert_eq!(state.deferred_cmd_queue, vec![5, 6]);
        assert!(state.event_queue.is_empty());
        assert!(state.response_queue.is_empty());
    }

    #[test]
    fn take_ready_commands_keeps_unready_deferred_entries() {
        let mut state = RuntimeState::<u8, u16, u32>::default();
        let ready_key = DeferredCommandKey {
            command_id: 1,
            command_signature: 10,
        };
        let waiting_key = DeferredCommandKey {
            command_id: 2,
            command_signature: 20,
        };
        state.enqueue_commands([3]);
        state.push_deferred_command(1);
        state.push_deferred_command(2);
        let _ = state.record_deferred_retry(waiting_key, 5, "wait");

        let batch = state.take_ready_commands(5, |command| match *command {
            1 => ready_key,
            2 => waiting_key,
            other => DeferredCommandKey {
                command_id: other as u64,
                command_signature: other as u64,
            },
        });

        assert_eq!(batch, vec![3, 1]);
        assert_eq!(state.deferred_cmd_queue, vec![2]);
    }

    #[test]
    fn deferred_helpers_track_retry_and_cleanup() {
        let mut state = RuntimeState::<u8, u16, u32>::default();
        let key = DeferredCommandKey {
            command_id: 10,
            command_signature: 20,
        };

        assert!(state.deferred_is_ready(&key, 0));
        let retry = state.record_deferred_retry(key, 12, "transient");
        assert_eq!(retry.attempts, 1);
        assert_eq!(retry.age_frames, 0);
        assert_eq!(retry.next_retry_frame, 13);
        assert!(state.deferred_contains(&key));
        assert!(!state.deferred_is_ready(&key, 12));
        assert!(state.deferred_is_ready(&key, 13));
        assert_eq!(
            state.clear_deferred_meta(&key).map(|meta| meta.attempts),
            Some(1)
        );
        assert!(!state.deferred_contains(&key));
    }

    #[test]
    fn render_bootstrap_plan_switches_between_create_and_reuse() {
        let target = PlatformRenderBootstrapTarget::new(
            9,
            glam::UVec2::new(1280, 720),
            PlatformRenderSurfaceKind::NativeWindow,
            PlatformSurfaceAlphaMode::Opaque,
            true,
        );

        assert_eq!(
            plan_render_bootstrap(false, target),
            RuntimeRenderBootstrapPlan {
                target,
                device_strategy: RenderBootstrapDeviceStrategy::CreateSharedDevice,
            }
        );
        assert_eq!(
            plan_render_bootstrap(true, target),
            RuntimeRenderBootstrapPlan {
                target,
                device_strategy: RenderBootstrapDeviceStrategy::ReuseSharedDevice,
            }
        );
    }
}
