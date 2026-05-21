use crate::core::audio::{process_audio_listener_binding, process_audio_source_bindings};
use crate::core::cmd::{deferred_command_key, engine_process_batch};
use crate::core::platforms::PlatformProxy;

#[cfg(target_arch = "wasm32")]
use js_sys::Date;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use super::GalfusResult;
use super::singleton::with_engine_singleton;

/// Main engine tick - processes events and updates state
pub fn galfus_tick(time: u64, delta_time: u32) -> GalfusResult {
    match with_engine_singleton(|engine| {
        engine.state.runtime.begin_tick(time, delta_time);
        engine.state.runtime.clear_events();

        // Reset profiling counters
        engine
            .state
            .profiling
            .begin_frame(delta_time, engine.state.runtime.frame_index());

        if engine.state.runtime.has_pending_commands() {
            // MARK: Command Processing
            #[cfg(not(target_arch = "wasm32"))]
            let cmd_start = Instant::now();
            #[cfg(target_arch = "wasm32")]
            let cmd_start = (Date::now() * 1_000_000.0) as u64;
            // Prefer newest host commands first; deferred retries are eventual and can be stale.
            let batch = engine
                .state
                .runtime
                .take_ready_commands(engine.state.runtime.frame_index(), |envelope| {
                    deferred_command_key(envelope.id, &envelope.cmd)
                });
            engine
                .state
                .runtime
                .set_had_commands_this_frame(!batch.is_empty());
            let result = engine_process_batch(&mut engine.state, &mut engine.platform, batch);
            #[cfg(not(target_arch = "wasm32"))]
            {
                engine.state.profiling.command.processing_ns =
                    cmd_start.elapsed().as_nanos() as u64;
            }
            #[cfg(target_arch = "wasm32")]
            {
                let now = (Date::now() * 1_000_000.0) as u64;
                engine.state.profiling.command.processing_ns = now.saturating_sub(cmd_start);
            }
            if result != GalfusResult::Success {
                return result;
            }
        }

        crate::core::target::refresh_target_indexes(&mut engine.state.universal_state);

        if engine.state.audio_available {
            process_audio_listener_binding(&mut engine.state);
            process_audio_source_bindings(&mut engine.state);
        }
        crate::core::resources::process_async_texture_results(&mut engine.state);
        if engine.state.audio_available {
            let audio_events = engine.state.audio.drain_events();
            for event in audio_events {
                engine
                    .state
                    .runtime
                    .push_event(crate::core::cmd::EngineEvent::System(
                        crate::core::system::events::SystemEvent::AudioReady {
                            resource_id: event.resource_id,
                            success: event.success,
                            message: event.message,
                        },
                    ));
            }
        }

        let events_before = engine.state.runtime.event_count();

        // MARK: Gamepad Processing
        engine.state.profiling.input.gamepad_processing_ns =
            engine.platform.process_gamepads(&mut engine.state);

        // MARK: Event Loop Pump
        engine.state.profiling.input.event_loop_pump_ns =
            engine.platform.pump_events(&mut engine.state);
        // vNext input policy: keep only global pointer stream.
        // Target-routed pointer relay is disabled.
        engine.state.profiling.ui.input_ns = 0;

        let events_after = engine.state.runtime.event_count();
        engine.state.profiling.input.total_events_dispatched = events_after - events_before;

        // MARK: Render Frame Lifecycle
        let frame_index = engine.state.runtime.advance_frame();
        for render_state in engine.state.render.states.values_mut() {
            render_state.begin_frame(frame_index);
        }

        // MARK: Request Redraw
        engine.state.profiling.render.request_redraw_ns = engine.platform.render(&mut engine.state);
        crate::core::profiling::metrics::refresh_runtime_metrics(&mut engine.state);
        engine.state.profiling.push_rolling_sample();
        GalfusResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
