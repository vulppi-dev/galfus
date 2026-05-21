use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyfunction]
fn galfus_init() -> u32 {
    galfus_core::galfus_init() as u32
}

#[pyfunction]
fn galfus_dispose() -> u32 {
    galfus_core::galfus_dispose() as u32
}

#[pyfunction]
fn galfus_send_queue(data: &[u8]) -> u32 {
    galfus_core::galfus_send_queue(data.as_ptr(), data.len()) as u32
}

fn take_bytes_result<F>(py: Python<'_>, receive_fn: F) -> PyResult<(Py<PyBytes>, u32)>
where
    F: FnOnce(*mut *const u8, *mut usize) -> u32,
{
    let mut length: usize = 0;
    let mut ptr: *const u8 = std::ptr::null();
    let length_ptr = &mut length as *mut usize;
    let ptr_ptr = &mut ptr as *mut *const u8;

    let result = receive_fn(ptr_ptr, length_ptr);
    if result != 0 || length == 0 {
        return Ok((PyBytes::new(py, &[]).into(), result));
    }

    let boxed = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
    let py_bytes = PyBytes::new(py, &boxed).into();
    Ok((py_bytes, result))
}

#[pyfunction]
fn galfus_receive_queue(py: Python<'_>) -> PyResult<(Py<PyBytes>, u32)> {
    take_bytes_result(py, |out_ptr, out_length| {
        galfus_core::galfus_receive_queue(out_ptr, out_length) as u32
    })
}

#[pyfunction]
fn galfus_receive_events(py: Python<'_>) -> PyResult<(Py<PyBytes>, u32)> {
    take_bytes_result(py, |out_ptr, out_length| {
        galfus_core::galfus_receive_events(out_ptr, out_length) as u32
    })
}

#[pyfunction]
fn galfus_upload_buffer(id: i64, upload_type: u32, data: &[u8]) -> u32 {
    galfus_core::galfus_upload_buffer(id as u64, upload_type, data.as_ptr(), data.len()) as u32
}

#[pyfunction]
fn galfus_tick(time: i64, delta_time: u32) -> u32 {
    galfus_core::galfus_tick(time as u64, delta_time) as u32
}

#[pyfunction]
fn galfus_get_profiling(py: Python<'_>) -> PyResult<(Py<PyBytes>, u32)> {
    take_bytes_result(py, |out_ptr, out_length| {
        galfus_core::galfus_get_profiling(out_ptr, out_length) as u32
    })
}

#[pymodule]
fn galfus(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(galfus_init, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_dispose, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_send_queue, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_receive_queue, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_receive_events, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_upload_buffer, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_tick, module)?)?;
    module.add_function(wrap_pyfunction!(galfus_get_profiling, module)?)?;
    Ok(())
}
