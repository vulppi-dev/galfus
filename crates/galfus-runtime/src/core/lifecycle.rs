use crate::core::platforms::DefaultPlatformProxy;
use std::thread;

use super::GalfusResult;
use super::cmd::EngineEvent;
use super::singleton::{ENGINE_INSTANCE, EngineSingleton, MAIN_THREAD_ID};
use super::state::EngineState;
use super::system::events::SystemEvent;

/// Initialize the engine (must be called from the main thread)
pub fn galfus_init() -> GalfusResult {
    let _ = env_logger::try_init();
    #[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
    if web_sys::window().is_none() {
        return GalfusResult::NotInBrowser;
    }
    let current_id = thread::current().id();

    if let Err(_) = MAIN_THREAD_ID.set(current_id) {
        if MAIN_THREAD_ID.get().unwrap() != &current_id {
            return GalfusResult::WrongThread;
        }
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_some() {
            return GalfusResult::AlreadyInitialized;
        } else {
            let platform = DefaultPlatformProxy::new();
            let mut state = EngineState::new();
            if let Err(message) = state.audio.init() {
                state.audio_available = false;
                state
                    .runtime
                    .push_event(EngineEvent::System(SystemEvent::Error {
                        scope: "audio-init".into(),
                        message: format!("Audio init failed, audio disabled: {message}"),
                        command_id: None,
                        command_type: None,
                    }));
            }
            *opt = Some(EngineSingleton { state, platform });
            return GalfusResult::Success;
        }
    })
}

/// Dispose of the engine and clean up resources
pub fn galfus_dispose() -> GalfusResult {
    let current_id = thread::current().id();

    if let Some(main_id) = MAIN_THREAD_ID.get() {
        if &current_id != main_id {
            return GalfusResult::WrongThread;
        }
    } else {
        return GalfusResult::NotInitialized;
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();

        // Explicitly clean up all render states before dropping
        if let Some(ref mut singleton) = *opt {
            for render_state in singleton.state.render.states.values_mut() {
                render_state.drop_all();
            }
            singleton.state.render.states.clear();
            // Clear all windows to drop GPU resources
            singleton.state.window.states.clear();
            singleton.state.window.window_id_map.clear();
        }

        *opt = None;
    });

    GalfusResult::Success
}
