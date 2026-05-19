#[cfg(target_arch = "wasm32")]
use js_sys::Date;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use super::GalfusResult;
use super::cmd::EngineBatchCmds;
use super::system::push_error_event;

#[cfg(target_arch = "wasm32")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}
use super::singleton::with_engine;

fn decode_engine_batch_cmds(data: &[u8]) -> Result<EngineBatchCmds, String> {
    galfus_protocol::decode_named(data)
        .map_err(|error| format!("Invalid MessagePack in command batch: {error}"))
}

/// Send a batch of commands to the engine
pub fn galfus_send_queue(ptr: *const u8, length: usize) -> GalfusResult {
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
            return GalfusResult::CmdInvalidMessagePackError;
        }
    };

    match with_engine(|engine| {
        engine.runtime.enqueue_commands(batch);
        GalfusResult::Success
    }) {
        Err(e) => return e,
        Ok(r) => r,
    }
}

/// Receive a batch of command responses from the engine
pub fn galfus_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> GalfusResult {
    match with_engine(|engine| {
        if engine.runtime.response_count() == 0 {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            engine.profiling.render.serialization_ns = 0;
            return GalfusResult::Success;
        }

        // MARK: Serialization
        #[cfg(not(target_arch = "wasm32"))]
        let serialization_start = Instant::now();
        #[cfg(target_arch = "wasm32")]
        let serialization_start = now_ns();
        let serialized_data = match galfus_protocol::encode_named(engine.runtime.response_batch()) {
            Ok(data) => data,
            Err(_) => return GalfusResult::UnknownError,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            engine.profiling.render.serialization_ns =
                serialization_start.elapsed().as_nanos() as u64;
        }
        #[cfg(target_arch = "wasm32")]
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
        GalfusResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

/// Receive a batch of spontaneous events from the engine
pub fn galfus_receive_events(out_ptr: *mut *const u8, out_length: *mut usize) -> GalfusResult {
    match with_engine(|engine| {
        if engine.runtime.event_count() == 0 {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            return GalfusResult::Success;
        }

        // MARK: Serialization
        #[cfg(not(target_arch = "wasm32"))]
        let serialization_start = Instant::now();
        #[cfg(target_arch = "wasm32")]
        let serialization_start = now_ns();
        let serialized_data = match galfus_protocol::encode_named(engine.runtime.event_batch()) {
            Ok(data) => data,
            Err(_) => return GalfusResult::UnknownError,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let serialization_time = serialization_start.elapsed().as_nanos() as u64;
        #[cfg(target_arch = "wasm32")]
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
        GalfusResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

#[cfg(test)]
#[path = "queue_tests.rs"]
mod tests;
