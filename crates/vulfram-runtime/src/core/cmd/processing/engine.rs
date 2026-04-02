use super::super::*;
use crate::core::VulframResult;
use crate::core::platforms::PlatformProxy;
use crate::core::state::EngineState;
pub(crate) fn engine_process_batch(
    engine: &mut EngineState,
    platform: &mut dyn PlatformProxy,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        let deferred_cmd = pack.cmd.clone();
        let response_count_before = engine.runtime.response_count();
        let command_id = pack.id;
        let command_type = super::defer::command_type_for_cmd(&deferred_cmd);
        let deferred_key = super::defer::deferred_command_key(command_id, &deferred_cmd);
        let was_deferred = engine.runtime.deferred_contains(&deferred_key);

        super::dispatch::dispatch_command(engine, platform, pack);

        if engine.runtime.response_count() <= response_count_before {
            continue;
        }
        let Some(last_response) = engine.runtime.last_response_cloned() else {
            continue;
        };

        if let Some((failure_kind, reason)) =
            super::defer::classify_failed_response(engine, &deferred_cmd, &last_response.response)
        {
            if failure_kind == super::defer::DeferredFailureKind::Transient {
                let retry = engine.runtime.record_deferred_retry(
                    deferred_key,
                    engine.runtime.frame_index(),
                    &reason,
                );

                if super::defer::should_drop_deferred(retry.attempts, retry.age_frames) {
                    let _ = engine.runtime.pop_response();
                    let dropped_reason = format!(
                        "deferred command dropped after {} attempts ({} frames): {}",
                        retry.attempts, retry.age_frames, reason
                    );
                    engine.runtime.push_response(CommandResponseEnvelope {
                        id: command_id,
                        response: super::response_maps::response_with_message(
                            last_response.response.clone(),
                            dropped_reason.clone(),
                        ),
                    });
                    super::defer::emit_dropped_event(
                        engine,
                        command_id,
                        command_type,
                        retry.attempts,
                        dropped_reason,
                    );
                    let _ = engine.runtime.clear_deferred_meta(&deferred_key);
                    if let Some(response) = engine
                        .runtime
                        .last_response()
                        .map(|entry| entry.response.clone())
                    {
                        super::error_events::maybe_emit_response_error_event(
                            engine, command_id, &response,
                        );
                    }
                } else {
                    let _ = engine.runtime.pop_response();
                    engine.runtime.push_deferred_command(EngineCmdEnvelope {
                        id: command_id,
                        cmd: deferred_cmd,
                    });
                    super::defer::emit_deferred_event(
                        engine,
                        command_id,
                        command_type,
                        retry.attempts,
                        format!("{reason} (next retry frame: {})", retry.next_retry_frame),
                    );
                }
                continue;
            }
        }

        if was_deferred && super::response_maps::response_is_success(&last_response.response) {
            let attempts = engine
                .runtime
                .clear_deferred_meta(&deferred_key)
                .map(|meta| meta.attempts)
                .unwrap_or(0);
            super::defer::emit_applied_event(engine, command_id, command_type, attempts);
        } else if was_deferred {
            let attempts = engine
                .runtime
                .clear_deferred_meta(&deferred_key)
                .map(|meta| meta.attempts)
                .unwrap_or(0);
            super::defer::emit_dropped_event(
                engine,
                command_id,
                command_type,
                attempts,
                super::response_maps::response_message(&last_response.response)
                    .unwrap_or_else(|| "deferred command ended without success".into()),
            );
        }

        super::error_events::maybe_emit_response_error_event(
            engine,
            command_id,
            &last_response.response,
        );
    }

    VulframResult::Success
}
