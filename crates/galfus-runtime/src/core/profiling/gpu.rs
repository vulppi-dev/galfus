use crate::core::profiling::state::TickProfiling;

pub use galfus_render::{GpuProfiler, GpuTimingReport};

pub fn apply_gpu_timing_report(profiling: &mut TickProfiling, report: GpuTimingReport) {
    profiling.gpu.shadow_ns = report.shadow_ns;
    profiling.gpu.light_cull_ns = report.light_cull_ns;
    profiling.gpu.forward_ns = report.forward_ns;
    profiling.gpu.compose_ns = report.compose_ns;
    profiling.gpu.total_ns = report.total_ns;
}
