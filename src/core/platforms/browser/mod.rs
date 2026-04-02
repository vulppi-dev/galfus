use crate::core::platform::{EventLoop, EventLoopProxy};
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::window::engine_cmd_window_create;
#[cfg(target_arch = "wasm32")]
use crate::core::window::engine_cmd_window_create_async;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};
use vulfram_platform::{browser_now_ns, should_poll_browser_gamepads};

use super::PlatformProxy;

pub mod input;

pub struct BrowserProxy {
    proxy: EventLoopProxy<EngineCustomEvents>,
}

impl BrowserProxy {
    pub fn new() -> Self {
        let proxy = EventLoop::<EngineCustomEvents>::with_user_event()
            .build()
            .unwrap()
            .create_proxy();
        Self { proxy }
    }

    fn now_ns() -> u64 {
        browser_now_ns(js_sys::Date::now())
    }
}

impl PlatformProxy for BrowserProxy {
    fn event_loop_proxy(&self) -> &EventLoopProxy<EngineCustomEvents> {
        &self.proxy
    }

    fn handle_window_create(
        &mut self,
        _state: &mut EngineState,
        _cmd_id: u64,
        args: &CmdWindowCreateArgs,
    ) -> Result<(), CmdResultWindowCreate> {
        #[cfg(target_arch = "wasm32")]
        {
            engine_cmd_window_create_async(args, _cmd_id)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let event_loop = ();
            let result = engine_cmd_window_create(_state, &event_loop, args);
            if result.success { Ok(()) } else { Err(result) }
        }
    }

    fn process_gamepads(&mut self, state: &mut EngineState) -> u64 {
        let start = Self::now_ns();
        let has_focus = web_sys::window()
            .and_then(|window| window.document())
            .map(|document| document.has_focus().unwrap_or(true))
            .unwrap_or(true);
        if !should_poll_browser_gamepads(!state.window.states.is_empty(), has_focus) {
            return Self::now_ns().saturating_sub(start);
        }
        crate::core::gamepad::process_web_gamepads(state);
        Self::now_ns().saturating_sub(start)
    }

    fn pump_events(&mut self, _state: &mut EngineState) -> u64 {
        0
    }

    fn render(&mut self, state: &mut EngineState) -> u64 {
        let start = Self::now_ns();
        crate::core::render::render_frames(state);
        Self::now_ns().saturating_sub(start)
    }
}
