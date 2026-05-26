#[unsafe(no_mangle)]
pub extern "C" fn galfus_init() -> u32 {
    galfus_core::galfus_init() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_dispose() -> u32 {
    galfus_core::galfus_dispose() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_send_queue(ptr: *const u8, length: usize) -> u32 {
    galfus_core::galfus_send_queue(ptr, length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    galfus_core::galfus_receive_queue(out_ptr, out_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_receive_events(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    galfus_core::galfus_receive_events(out_ptr, out_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_upload_buffer(
    bfr_id: u64,
    upload_type: u32,
    bfr_ptr: *const u8,
    bfr_length: usize,
) -> u32 {
    galfus_core::galfus_upload_buffer(bfr_id, upload_type, bfr_ptr, bfr_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_tick(time: i64, delta_time: u32) -> u32 {
    galfus_core::galfus_tick(time, delta_time) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn galfus_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    galfus_core::galfus_get_profiling(out_ptr, out_length) as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn tick_signature_uses_i64() {
        let _fn_ptr: extern "C" fn(i64, u32) -> u32 = super::galfus_tick;
    }
}
