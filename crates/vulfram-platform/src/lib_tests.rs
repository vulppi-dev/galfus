use super::{
    INPUT_REDRAW_GRACE_MS, PlatformActivityEvent, RedrawContext, WindowRedrawInput,
    active_windows_from_events, plan_window_redraws,
};
use std::collections::HashSet;

#[test]
fn active_windows_from_events_deduplicates_window_ids() {
    let windows = active_windows_from_events(&[
        PlatformActivityEvent::Pointer { window_id: 2 },
        PlatformActivityEvent::Keyboard { window_id: 2 },
        PlatformActivityEvent::Window { window_id: 3 },
    ]);
    assert_eq!(windows.len(), 2);
    assert!(windows.contains(&2));
    assert!(windows.contains(&3));
}

#[test]
fn redraw_plan_extends_input_grace_and_requests_redraw() {
    let plans = plan_window_redraws(
        &[PlatformActivityEvent::Pointer { window_id: 7 }],
        &HashSet::new(),
        &[WindowRedrawInput {
            window_id: 7,
            redraw_force_until_ms: 0,
            is_dirty: false,
        }],
        RedrawContext {
            now_ms: 1000,
            had_commands_this_frame: false,
            has_ui_animations: false,
            has_ui_repaint_request: false,
            has_pending_texture_work: false,
            has_unbound_ui_async_loading: false,
        },
    );
    assert_eq!(plans[0].redraw_force_until_ms, 1000 + INPUT_REDRAW_GRACE_MS);
    assert!(plans[0].should_request_redraw);
}

#[test]
fn redraw_plan_respects_pending_ui_windows_without_input() {
    let mut pending = HashSet::new();
    pending.insert(9);
    let plans = plan_window_redraws(
        &[],
        &pending,
        &[WindowRedrawInput {
            window_id: 9,
            redraw_force_until_ms: 0,
            is_dirty: false,
        }],
        RedrawContext {
            now_ms: 100,
            had_commands_this_frame: false,
            has_ui_animations: false,
            has_ui_repaint_request: false,
            has_pending_texture_work: false,
            has_unbound_ui_async_loading: false,
        },
    );
    assert!(plans[0].should_request_redraw);
}
