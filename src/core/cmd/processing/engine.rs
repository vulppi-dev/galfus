use super::super::*;
use crate::core::VulframResult;
use crate::core::platforms::PlatformProxy;
use crate::core::state::EngineState;
use vulfram_runtime::DeferredCommandMeta;

pub(crate) fn engine_process_batch(
    engine: &mut EngineState,
    platform: &mut dyn PlatformProxy,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        let deferred_cmd = pack.cmd.clone();
        let response_count_before = engine.runtime.response_queue.len();
        let command_id = pack.id;
        let command_type = super::defer::command_type_for_cmd(&deferred_cmd);
        let deferred_key = super::defer::deferred_command_key(command_id, &deferred_cmd);
        let was_deferred = engine.runtime.deferred_cmd_meta.contains_key(&deferred_key);

        super::dispatch::dispatch_command(engine, platform, pack);

        if engine.runtime.response_queue.len() <= response_count_before {
            continue;
        }
        let Some(last_response) = engine.runtime.response_queue.last().cloned() else {
            continue;
        };

        if let Some((failure_kind, reason)) =
            super::defer::classify_failed_response(engine, &deferred_cmd, &last_response.response)
        {
            if failure_kind == super::defer::DeferredFailureKind::Transient {
                let (attempts, age_frames, next_retry_frame) = {
                    let meta = engine
                        .runtime
                        .deferred_cmd_meta
                        .entry(deferred_key)
                        .or_insert_with(|| DeferredCommandMeta {
                            first_frame: engine.runtime.frame.frame_index,
                            attempts: 0,
                            next_retry_frame: engine.runtime.frame.frame_index,
                            last_reason: String::new(),
                        });
                    meta.attempts = meta.attempts.saturating_add(1);
                    meta.last_reason = reason.clone();
                    let backoff = super::defer::defer_backoff_frames(meta.attempts);
                    meta.next_retry_frame =
                        engine.runtime.frame.frame_index.saturating_add(backoff);
                    (
                        meta.attempts,
                        engine
                            .runtime
                            .frame
                            .frame_index
                            .saturating_sub(meta.first_frame),
                        meta.next_retry_frame,
                    )
                };

                if super::defer::should_drop_deferred(attempts, age_frames) {
                    let _ = engine.runtime.response_queue.pop();
                    let dropped_reason = format!(
                        "deferred command dropped after {} attempts ({} frames): {}",
                        attempts, age_frames, reason
                    );
                    engine.runtime.response_queue.push(CommandResponseEnvelope {
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
                        attempts,
                        dropped_reason,
                    );
                    engine.runtime.deferred_cmd_meta.remove(&deferred_key);
                    if let Some(response) = engine
                        .runtime
                        .response_queue
                        .last()
                        .map(|entry| entry.response.clone())
                    {
                        super::error_events::maybe_emit_response_error_event(
                            engine, command_id, &response,
                        );
                    }
                } else {
                    let _ = engine.runtime.response_queue.pop();
                    engine.runtime.deferred_cmd_queue.push(EngineCmdEnvelope {
                        id: command_id,
                        cmd: deferred_cmd,
                    });
                    super::defer::emit_deferred_event(
                        engine,
                        command_id,
                        command_type,
                        attempts,
                        format!("{reason} (next retry frame: {next_retry_frame})"),
                    );
                }
                continue;
            }
        }

        if was_deferred && super::response_maps::response_is_success(&last_response.response) {
            let attempts = engine
                .runtime
                .deferred_cmd_meta
                .remove(&deferred_key)
                .map(|meta| meta.attempts)
                .unwrap_or(0);
            super::defer::emit_applied_event(engine, command_id, command_type, attempts);
        } else if was_deferred {
            let attempts = engine
                .runtime
                .deferred_cmd_meta
                .remove(&deferred_key)
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
