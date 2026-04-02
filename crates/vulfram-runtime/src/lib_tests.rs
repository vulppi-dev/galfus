use super::{
    DEFER_MAX_AGE_FRAMES, DEFER_MAX_ATTEMPTS, DeferredCommandKey, RenderBootstrapDeviceStrategy,
    RuntimeFrameState, RuntimeRenderBootstrapPlan, RuntimeState, defer_backoff_frames,
    plan_render_bootstrap, should_drop_deferred,
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
