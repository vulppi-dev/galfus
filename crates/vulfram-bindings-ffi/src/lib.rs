#[unsafe(no_mangle)]
pub extern "C" fn vulfram_init() -> u32 {
    vulfram_core::vulfram_init() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_dispose() -> u32 {
    vulfram_core::vulfram_dispose() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_send_queue(ptr: *const u8, length: usize) -> u32 {
    vulfram_core::vulfram_send_queue(ptr, length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    vulfram_core::vulfram_receive_queue(out_ptr, out_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_receive_events(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    vulfram_core::vulfram_receive_events(out_ptr, out_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_upload_buffer(
    bfr_id: u64,
    upload_type: u32,
    bfr_ptr: *const u8,
    bfr_length: usize,
) -> u32 {
    vulfram_core::vulfram_upload_buffer(bfr_id, upload_type, bfr_ptr, bfr_length) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_tick(time: u64, delta_time: u32) -> u32 {
    vulfram_core::vulfram_tick(time, delta_time) as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn vulfram_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
    vulfram_core::vulfram_get_profiling(out_ptr, out_length) as u32
}
