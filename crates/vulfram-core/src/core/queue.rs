#[cfg(feature = "wasm")]
use js_sys::Date;
#[cfg(not(feature = "wasm"))]
use std::time::Instant;

use super::VulframResult;
use super::cmd::EngineBatchCmds;
use super::system::push_error_event;

#[cfg(feature = "wasm")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}
use super::singleton::with_engine;

fn decode_engine_batch_cmds(data: &[u8]) -> Result<EngineBatchCmds, String> {
    vulfram_protocol::decode_named(data)
        .map_err(|error| format!("Invalid MessagePack in command batch: {error}"))
}

/// Send a batch of commands to the engine
pub fn vulfram_send_queue(ptr: *const u8, length: usize) -> VulframResult {
    let data = unsafe { std::slice::from_raw_parts(ptr, length) };

    let batch = match decode_engine_batch_cmds(data) {
        Ok(batch) => batch,
        Err(message) => {
            let _ = with_engine(|engine| {
                push_error_event(
                    engine,
                    "serialization",
                    message,
                    None,
                    Some("send-queue".into()),
                );
            });
            return VulframResult::CmdInvalidMessagePackError;
        }
    };

    match with_engine(|engine| {
        engine.runtime.enqueue_commands(batch);
        VulframResult::Success
    }) {
        Err(e) => return e,
        Ok(r) => r,
    }
}

/// Receive a batch of command responses from the engine
pub fn vulfram_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        if engine.runtime.response_count() == 0 {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            engine.profiling.render.serialization_ns = 0;
            return VulframResult::Success;
        }

        // MARK: Serialization
        #[cfg(not(feature = "wasm"))]
        let serialization_start = Instant::now();
        #[cfg(feature = "wasm")]
        let serialization_start = now_ns();
        let serialized_data = match vulfram_protocol::encode_named(engine.runtime.response_batch())
        {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };
        #[cfg(not(feature = "wasm"))]
        {
            engine.profiling.render.serialization_ns =
                serialization_start.elapsed().as_nanos() as u64;
        }
        #[cfg(feature = "wasm")]
        {
            engine.profiling.render.serialization_ns = now_ns().saturating_sub(serialization_start);
        }

        let data_length = serialized_data.len();

        // Transfer ownership via Box::into_raw (zero-copy)
        let boxed = serialized_data.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u8;

        unsafe {
            *out_ptr = ptr;
            *out_length = data_length;
        }

        engine.runtime.clear_responses();
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

/// Receive a batch of spontaneous events from the engine
pub fn vulfram_receive_events(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        if engine.runtime.event_count() == 0 {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            return VulframResult::Success;
        }

        // MARK: Serialization
        #[cfg(not(feature = "wasm"))]
        let serialization_start = Instant::now();
        #[cfg(feature = "wasm")]
        let serialization_start = now_ns();
        let serialized_data = match vulfram_protocol::encode_named(engine.runtime.event_batch()) {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };
        #[cfg(not(feature = "wasm"))]
        let serialization_time = serialization_start.elapsed().as_nanos() as u64;
        #[cfg(feature = "wasm")]
        let serialization_time = now_ns().saturating_sub(serialization_start);

        // Only update profiling if we're serializing responses too
        // (to avoid overwriting response serialization time)
        if engine.profiling.render.serialization_ns == 0 {
            engine.profiling.render.serialization_ns = serialization_time;
        } else {
            engine.profiling.render.serialization_ns += serialization_time;
        }

        let data_length = serialized_data.len();

        // Transfer ownership via Box::into_raw (zero-copy)
        let boxed = serialized_data.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u8;

        unsafe {
            *out_ptr = ptr;
            *out_length = data_length;
        }

        engine.runtime.clear_events();
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HostCmdWindowCloseArgs {
        window_id: u32,
    }

    #[derive(Serialize)]
    struct HostInvalidEnvelope<'a> {
        id: &'a str,
        #[serde(rename = "type")]
        command_type: &'a str,
        content: HostCmdWindowCloseArgs,
    }

    #[test]
    fn send_queue_invalid_type_emits_serialization_error_with_path() {
        let payload = rmp_serde::to_vec_named(&vec![HostInvalidEnvelope {
            id: "invalid-id-type",
            command_type: "cmd-window-close",
            content: HostCmdWindowCloseArgs { window_id: 1 },
        }])
        .expect("host payload serialization must succeed");

        let error = decode_engine_batch_cmds(&payload)
            .expect_err("invalid payload should produce decode error");
        assert!(
            error.contains("at '"),
            "serialization message should include decode path: {error}"
        );
        assert!(
            error.contains("id"),
            "serialization message should mention failing field: {error}"
        );
    }
}
