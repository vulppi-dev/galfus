use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct BufferResult {
    pub buffer: Buffer,
    pub result: u32,
}

fn take_buffer_result<F>(receive_fn: F) -> BufferResult
where
    F: FnOnce(*mut *const u8, *mut usize) -> u32,
{
    let mut length: usize = 0;
    let mut ptr: *const u8 = std::ptr::null();
    let length_ptr = &mut length as *mut usize;
    let ptr_ptr = &mut ptr as *mut *const u8;

    let result = receive_fn(ptr_ptr, length_ptr);
    if result != 0 || length == 0 {
        return BufferResult {
            buffer: Buffer::from(vec![]),
            result,
        };
    }

    let boxed = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
    let buffer = Buffer::from(boxed.into_vec());
    BufferResult { buffer, result }
}

#[napi]
pub fn galfus_init() -> u32 {
    galfus_core::galfus_init() as u32
}

#[napi]
pub fn galfus_dispose() -> u32 {
    galfus_core::galfus_dispose() as u32
}

#[napi]
pub fn galfus_send_queue(data: Buffer) -> u32 {
    galfus_core::galfus_send_queue(data.as_ptr(), data.len()) as u32
}

#[napi]
pub fn galfus_receive_queue() -> Result<BufferResult> {
    Ok(take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_receive_queue(out_ptr, out_length) as u32
    }))
}

#[napi]
pub fn galfus_receive_events() -> Result<BufferResult> {
    Ok(take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_receive_events(out_ptr, out_length) as u32
    }))
}

#[napi]
pub fn galfus_upload_buffer(id: i64, upload_type: u32, data: Buffer) -> u32 {
    galfus_core::galfus_upload_buffer(id as u64, upload_type, data.as_ptr(), data.len()) as u32
}

#[napi]
pub fn galfus_tick(time: i64, delta_time: u32) -> u32 {
    galfus_core::galfus_tick(time as u64, delta_time) as u32
}

#[napi]
pub fn galfus_get_profiling() -> Result<BufferResult> {
    Ok(take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_get_profiling(out_ptr, out_length) as u32
    }))
}
