use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::{CommandResponse, EngineCmd, EngineEvent};
use galfus_core::core::window::CmdWindowCreateArgs;
use galfus_core::core::window::WindowEvent;
use glam::UVec2;
use std::time::Duration;

use crate::demo::io::{receive_events, receive_responses, send_commands};
use galfus_core::core;

const WINDOW_READY_TIMEOUT: Duration = Duration::from_secs(10);
const WINDOW_READY_DELTA_MS: u32 = 16;

#[derive(Debug, Clone, Copy)]
pub struct WindowBinding {
    pub realm_id: u32,
}

pub fn create_window(window_id: u32, title: &str) -> WindowBinding {
    let create_cmd = EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
        window_id,
        title: title.into(),
        size: UVec2::new(640, 360),
        resizable: true,
        initial_state: galfus_core::core::window::EngineWindowState::Windowed,
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), GalfusResult::Success);
    wait_for_window_ready(window_id, title)
}

fn wait_for_window_ready(window_id: u32, title: &str) -> WindowBinding {
    let start = std::time::Instant::now();
    let mut total_ms: u64 = 0;
    let mut realm_id: Option<u32> = None;
    let mut got_window_signal = false;
    let mut observed_events: Vec<String> = Vec::new();

    while start.elapsed() < WINDOW_READY_TIMEOUT {
        assert_eq!(
            core::galfus_tick(total_ms as i64, WINDOW_READY_DELTA_MS),
            GalfusResult::Success
        );
        total_ms = total_ms.saturating_add(WINDOW_READY_DELTA_MS as u64);

        for response in receive_responses() {
            if let CommandResponse::WindowCreate(res) = response.response {
                if !res.success {
                    panic!("Window creation failed for '{}': {}", title, res.message);
                }
                realm_id = res.realm_id;
                if realm_id.is_none() {
                    panic!("Window '{}' create response missing realm_id", title);
                }
            }
        }

        for event in receive_events() {
            if let EngineEvent::Window(WindowEvent::OnCreate {
                window_id: created_window_id,
            }) = event
            {
                if created_window_id == window_id {
                    got_window_signal = true;
                }
            } else if let EngineEvent::Window(WindowEvent::OnRedrawRequest {
                window_id: redraw_window_id,
            }) = event
            {
                if redraw_window_id == window_id {
                    got_window_signal = true;
                }
            } else if let EngineEvent::Window(WindowEvent::OnFocus {
                window_id: focused_window_id,
                ..
            }) = event
            {
                if focused_window_id == window_id {
                    got_window_signal = true;
                }
            } else if observed_events.len() < 8 {
                observed_events.push(format!("{:?}", event));
            }
        }

        if let Some(realm_id) = realm_id {
            if got_window_signal {
                return WindowBinding { realm_id };
            }
        }
    }

    panic!(
        "Window readiness timeout for '{}' (window_id={}, timeout_ms={}): response_realm_id={:?}, on_create_event={}, observed_events={:?}",
        title,
        window_id,
        WINDOW_READY_TIMEOUT.as_millis(),
        realm_id,
        got_window_signal,
        observed_events
    );
}
