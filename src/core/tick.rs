use crate::core::audio::{process_audio_listener_binding, process_audio_source_bindings};
use crate::core::cmd::{deferred_command_key, engine_process_batch};
use crate::core::platforms::PlatformProxy;

#[cfg(feature = "wasm")]
use js_sys::Date;
#[cfg(not(feature = "wasm"))]
use std::time::Instant;

use super::VulframResult;
use super::singleton::with_engine_singleton;

/// Main engine tick - processes events and updates state
pub fn vulfram_tick(time: u64, delta_time: u32) -> VulframResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;
        engine.state.event_queue.clear();

        // Reset profiling counters
        engine
            .state
            .profiling
            .begin_frame(delta_time, engine.state.frame_index);

        if !engine.state.deferred_cmd_queue.is_empty() || !engine.state.cmd_queue.is_empty() {
            // MARK: Command Processing
            #[cfg(not(feature = "wasm"))]
            let cmd_start = Instant::now();
            #[cfg(feature = "wasm")]
            let cmd_start = (Date::now() * 1_000_000.0) as u64;
            let deferred = std::mem::take(&mut engine.state.deferred_cmd_queue);
            // Prefer newest host commands first; deferred retries are eventual and can be stale.
            let mut batch = std::mem::take(&mut engine.state.cmd_queue);
            let mut still_deferred = Vec::new();
            for envelope in deferred {
                let key = deferred_command_key(envelope.id, &envelope.cmd);
                let ready = engine
                    .state
                    .deferred_cmd_meta
                    .get(&key)
                    .map(|meta| meta.next_retry_frame <= engine.state.frame_index)
                    .unwrap_or(true);
                if ready {
                    batch.push(envelope);
                } else {
                    still_deferred.push(envelope);
                }
            }
            engine.state.deferred_cmd_queue = still_deferred;
            let result = engine_process_batch(&mut engine.state, &mut engine.platform, batch);
            #[cfg(not(feature = "wasm"))]
            {
                engine.state.profiling.command.processing_ns =
                    cmd_start.elapsed().as_nanos() as u64;
            }
            #[cfg(feature = "wasm")]
            {
                let now = (Date::now() * 1_000_000.0) as u64;
                engine.state.profiling.command.processing_ns = now.saturating_sub(cmd_start);
            }
            if result != VulframResult::Success {
                return result;
            }
        }

        crate::core::target::refresh_target_indexes(&mut engine.state.universal_state);

        if engine.state.audio_available {
            process_audio_listener_binding(&mut engine.state);
            process_audio_source_bindings(&mut engine.state);
        }
        crate::core::resources::process_async_texture_results(&mut engine.state);
        crate::core::ui::cmd::process_async_ui_image_results(&mut engine.state);
        if engine.state.audio_available {
            let audio_events = engine.state.audio.drain_events();
            for event in audio_events {
                engine
                    .state
                    .event_queue
                    .push(crate::core::cmd::EngineEvent::System(
                        crate::core::system::events::SystemEvent::AudioReady {
                            resource_id: event.resource_id,
                            success: event.success,
                            message: event.message,
                        },
                    ));
            }
        }

        let events_before = engine.state.event_queue.len();

        // MARK: Gamepad Processing
        engine.state.profiling.input.gamepad_processing_ns =
            engine.platform.process_gamepads(&mut engine.state);

        // MARK: Event Loop Pump
        engine.state.profiling.input.event_loop_pump_ns =
            engine.platform.pump_events(&mut engine.state);
        crate::core::input::route_pointer_events(&mut engine.state);
        #[cfg(not(feature = "wasm"))]
        let ui_input_start = Instant::now();
        #[cfg(feature = "wasm")]
        let ui_input_start = (Date::now() * 1_000_000.0) as u64;
        crate::core::ui::input::process_ui_input(&mut engine.state);
        #[cfg(not(feature = "wasm"))]
        {
            engine.state.profiling.ui.input_ns = ui_input_start.elapsed().as_nanos() as u64;
        }
        #[cfg(feature = "wasm")]
        {
            let now = (Date::now() * 1_000_000.0) as u64;
            engine.state.profiling.ui.input_ns = now.saturating_sub(ui_input_start);
        }

        let events_after = engine.state.event_queue.len();
        engine.state.profiling.input.total_events_dispatched = events_after - events_before;

        // MARK: Render Frame Lifecycle
        engine.state.frame_index = engine.state.frame_index.wrapping_add(1);
        let frame_index = engine.state.frame_index;
        for render_state in engine.state.render.states.values_mut() {
            render_state.begin_frame(frame_index);
        }

        // MARK: Request Redraw
        engine.state.profiling.render.request_redraw_ns = engine.platform.render(&mut engine.state);
        engine.state.profiling.push_rolling_sample();
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
