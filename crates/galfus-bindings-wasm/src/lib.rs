use console_error_panic_hook::set_once;
use wasm_bindgen::prelude::*;

#[inline]
fn ensure_panic_hook() {
    set_once();
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    ensure_panic_hook();
}

#[wasm_bindgen]
pub struct BufferResult {
    buffer: Vec<u8>,
    result: u32,
}

#[wasm_bindgen]
impl BufferResult {
    #[wasm_bindgen(js_name = takeBuffer)]
    pub fn take_buffer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }

    #[wasm_bindgen(getter)]
    pub fn result(&self) -> u32 {
        self.result
    }
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
            buffer: Vec::new(),
            result,
        };
    }

    let boxed = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
    BufferResult {
        buffer: boxed.into_vec(),
        result,
    }
}

#[wasm_bindgen]
pub fn galfus_init() -> u32 {
    ensure_panic_hook();
    galfus_core::galfus_init() as u32
}

#[wasm_bindgen]
pub fn galfus_dispose() -> u32 {
    ensure_panic_hook();
    galfus_core::galfus_dispose() as u32
}

#[wasm_bindgen]
pub fn galfus_send_queue(data: &[u8]) -> u32 {
    ensure_panic_hook();
    galfus_core::galfus_send_queue(data.as_ptr(), data.len()) as u32
}

#[wasm_bindgen]
pub fn galfus_receive_queue() -> BufferResult {
    ensure_panic_hook();
    take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_receive_queue(out_ptr, out_length) as u32
    })
}

#[wasm_bindgen]
pub fn galfus_receive_events() -> BufferResult {
    ensure_panic_hook();
    take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_receive_events(out_ptr, out_length) as u32
    })
}

#[wasm_bindgen]
pub fn galfus_upload_buffer(id: u64, upload_type: u32, data: &[u8]) -> u32 {
    ensure_panic_hook();
    galfus_core::galfus_upload_buffer(id, upload_type, data.as_ptr(), data.len()) as u32
}

#[wasm_bindgen]
pub fn galfus_tick(time_ms: f64, delta_ms: u32) -> u32 {
    ensure_panic_hook();
    galfus_core::galfus_tick(time_ms as u64, delta_ms) as u32
}

#[wasm_bindgen]
pub fn galfus_get_profiling() -> BufferResult {
    ensure_panic_hook();
    take_buffer_result(|out_ptr, out_length| {
        galfus_core::galfus_get_profiling(out_ptr, out_length) as u32
    })
}
