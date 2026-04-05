use std::time::{Duration, Instant};
use vulfram_core::core::VulframResult;
use vulfram_core::core::cmd::EngineEvent;
use vulfram_core::core::input::events::{ElementState, KeyboardEvent};
use vulfram_core::core::input::keycodes::{KEY_ESCAPE, KEY_W};
use vulfram_core::core::window::{CmdWindowCloseArgs, WindowEvent};
use vulfram_core::vulfram_tick;

use crate::demo::io::{receive_events, receive_responses, send_commands};
use vulfram_core::core::cmd::EngineCmd;

pub fn run_loop_with_events<F, G>(
    window_id: u32,
    max_duration: Option<Duration>,
    mut on_frame: F,
    mut on_event: G,
) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
    G: FnMut(EngineEvent) -> bool,
{
    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;
    let target_frame_time = Duration::from_millis(16);

    loop {
        if let Some(max_duration) = max_duration {
            if start_time.elapsed() >= max_duration {
                return false;
            }
        }

        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        let frame_cmds = on_frame(total_ms, delta_ms);
        if !frame_cmds.is_empty() {
            let _ = send_commands(frame_cmds);
        }

        let tick_start = Instant::now();
        assert_eq!(vulfram_tick(total_ms, delta_ms), VulframResult::Success);

        let _ = receive_responses();
        if handle_events(window_id, &mut on_event) {
            let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                window_id,
            })]);
            return true;
        }

        let elapsed = tick_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }
}

fn is_close_event(window_id: u32, event: &EngineEvent) -> bool {
    match event {
        EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id }) if *id == window_id => {
            true
        }
        EngineEvent::Keyboard(KeyboardEvent::OnInput {
            window_id: id,
            key_code,
            state: ElementState::Pressed,
            modifiers,
            ..
        }) if *id == window_id => {
            // Escape always closes demo windows; Ctrl+W is the explicit close shortcut.
            *key_code == KEY_ESCAPE || (*key_code == KEY_W && modifiers.ctrl)
        }
        _ => false,
    }
}

fn handle_events<F>(window_id: u32, on_event: &mut F) -> bool
where
    F: FnMut(EngineEvent) -> bool,
{
    let events = receive_events();
    for event in events {
        let should_close = on_event(event.clone());
        if should_close || is_close_event(window_id, &event) {
            return true;
        }
    }
    false
}
