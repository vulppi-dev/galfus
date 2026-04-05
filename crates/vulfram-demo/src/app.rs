use crate::demo::{DemoContext, create_window, run_demo, select_demo};
use std::sync::Mutex;
use vulfram_core::core::VulframResult;
use vulfram_core::core::cmd::EngineCmd;
use vulfram_core::core::window::CmdWindowCloseArgs;

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

pub fn run() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(vulfram_core::core::vulfram_init(), VulframResult::Success);

    let demo_kind = select_demo();
    let window_id: u32 = 1;

    let title = demo_kind.title();
    let binding = create_window(window_id, &title);

    let close_sent = run_demo(
        demo_kind,
        DemoContext {
            window_id,
            realm_id: binding.realm_id,
        },
    );

    if !close_sent {
        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
        let _ = crate::demo::send_commands(vec![close_cmd]);
    }

    assert_eq!(
        vulfram_core::core::vulfram_dispose(),
        VulframResult::Success
    );
}
