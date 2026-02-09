use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::window::CmdWindowCreateArgs;
use glam::UVec2;
use std::time::Duration;

use crate::core;
use crate::demo::io::{receive_responses, send_commands};

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
    pump_for(Duration::from_millis(200));
    wait_for_confirmation(window_id)
}

fn pump_for(duration: Duration) {
    let start = std::time::Instant::now();
    let mut total_ms = 0u64;
    while start.elapsed() < duration {
        assert_eq!(core::vulfram_tick(total_ms, 16), VulframResult::Success);
        total_ms += 16;
        std::thread::sleep(Duration::from_millis(16));
    }
}

fn wait_for_confirmation(_window_id: u32) -> WindowBinding {
    for _ in 0..100 {
        let responses = receive_responses();
        for response in responses {
            match response.response {
                CommandResponse::WindowCreate(res) => {
                    if res.success {
                        return WindowBinding {
                            realm_id: res.realm_id.expect("window response missing realm_id"),
                        };
                    } else {
                        panic!("Window creation failed: {}", res.message);
                    }
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(core::vulfram_tick(0, 0), VulframResult::Success);
    }
    panic!("Window creation did not complete in time");
}
