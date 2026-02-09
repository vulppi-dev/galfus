mod core;
mod demo;

use crate::core::VulframResult;
use crate::core::cmd::EngineCmd;
use crate::core::window::CmdWindowCloseArgs;
use crate::demo::{DemoContext, create_window, run_demo, select_demo};
use std::sync::Mutex;

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(core::vulfram_init(), VulframResult::Success);

    let demo_kind = select_demo();
    let window_id: u32 = 1;

    let binding = create_window(window_id, demo_kind.title());

    let close_sent = run_demo(
        demo_kind,
        DemoContext {
            window_id,
            realm_id: binding.realm_id,
        },
    );

    if !close_sent {
        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
        let _ = demo::send_commands(vec![close_cmd]);
    }

    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
}
