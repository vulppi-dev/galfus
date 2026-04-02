mod core;

pub use core::{
    vulfram_dispose, vulfram_get_profiling, vulfram_init, vulfram_receive_events,
    vulfram_receive_queue, vulfram_send_queue, vulfram_tick, vulfram_upload_buffer,
};

// ============================================================================
// Python Exports - for Python bindings via PyO3
// ============================================================================
#[cfg(feature = "python")]
#[allow(unused)]
mod python_exports {
    use super::core;
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    #[pyfunction]
    fn vulfram_init() -> u32 {
        core::vulfram_init() as u32
    }

    #[pyfunction]
    fn vulfram_dispose() -> u32 {
        core::vulfram_dispose() as u32
    }

    #[pyfunction]
    fn vulfram_send_queue(data: &[u8]) -> u32 {
        core::vulfram_send_queue(data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn vulfram_receive_queue(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pyfunction]
    fn vulfram_receive_events(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pyfunction]
    fn vulfram_upload_buffer(id: i64, upload_type: u32, data: &[u8]) -> u32 {
        core::vulfram_upload_buffer(id as u64, upload_type, data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn vulfram_tick(time: i64, delta_time: u32) -> u32 {
        core::vulfram_tick(time as u64, delta_time) as u32
    }

    #[pyfunction]
    fn vulfram_get_profiling(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pymodule]
    fn vulfram(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(vulfram_init, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_dispose, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_send_queue, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_receive_queue, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_receive_events, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_upload_buffer, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_tick, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_get_profiling, module)?)?;
        Ok(())
    }
}
