use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd, EngineEvent};
use crate::core::window::CmdWindowCreateArgs;
use crate::core::window::WindowEvent;
use glam::UVec2;
use std::time::Duration;

use crate::core;
use crate::demo::io::{receive_events, receive_responses, send_commands};

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
        size: UVec2::new(1280, 720),
        resizable: true,
        initial_state: crate::core::window::EngineWindowState::Maximized,
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);
    wait_for_window_ready(window_id, title)
}

fn wait_for_window_ready(window_id: u32, title: &str) -> WindowBinding {
    let start = std::time::Instant::now();
    let mut total_ms: u64 = 0;
    let mut realm_id: Option<u32> = None;
    let mut got_window_create_event = false;
    let mut observed_events: Vec<String> = Vec::new();

    while start.elapsed() < WINDOW_READY_TIMEOUT {
        assert_eq!(
            core::vulfram_tick(total_ms, WINDOW_READY_DELTA_MS),
            VulframResult::Success
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
                    got_window_create_event = true;
                }
            } else if observed_events.len() < 8 {
                observed_events.push(format!("{:?}", event));
            }
        }

        if let Some(realm_id) = realm_id {
            if got_window_create_event {
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
        got_window_create_event,
        observed_events
    );
}
