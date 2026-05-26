use serde::{Deserialize, Serialize};

use crate::core::GalfusResult;
use crate::core::profiling::state::{FrameProfilingSample, ProfilingDetailLevel, TickProfiling};
use crate::core::singleton::with_engine;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingDomainUs {
    pub command: f64,
    pub input: f64,
    pub routing: f64,
    pub render: f64,
    pub gpu: f64,
    pub graph: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingTimesUs {
    pub command_processing: f64,
    pub gamepad_processing: f64,
    pub event_loop_pump: f64,
    pub request_redraw: f64,
    pub serialization: f64,
    pub render_total: f64,
    pub render_shadow: f64,
    pub render_windows: f64,
    pub frame_delta: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingCounters {
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingRollingWindow {
    pub sample_count: usize,
    pub command_us_avg: f64,
    pub input_us_avg: f64,
    pub render_us_avg: f64,
    pub gpu_us_avg: f64,
    pub fps_avg: f64,
    pub frame_us_p50: f64,
    pub frame_us_p95: f64,
    pub frame_us_p99: f64,
    pub frame_us_max: f64,
    pub render_us_p95: f64,
    pub gpu_us_p95: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingMemoryBytes {
    pub ram_current: u64,
    pub ram_peak: u64,
    pub gpu_current: u64,
    pub gpu_peak: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingUtilization {
    pub cpu_percent: f32,
    pub gpu_frame_percent: f32,
    pub command_percent: f32,
    pub input_percent: f32,
    pub render_percent: f32,
    pub graph_percent: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingCache {
    pub render_pipeline_hits: u32,
    pub render_pipeline_misses: u32,
    pub render_pipeline_evictions: u32,
    pub compute_pipeline_hits: u32,
    pub compute_pipeline_misses: u32,
    pub compute_pipeline_evictions: u32,
    pub compose_bind_cache_hits: u32,
    pub compose_bind_cache_misses: u32,
    pub compose_bind_cache_evictions: u32,
    pub post_bind_cache_hits: u32,
    pub post_bind_cache_misses: u32,
    pub post_bind_cache_evictions: u32,
    pub material_shader_module_evictions: u32,
    pub material_program_cache_evictions: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingData {
    pub times_us: ProfilingTimesUs,
    pub domain_us: ProfilingDomainUs,
    pub counters: ProfilingCounters,
    pub rolling: ProfilingRollingWindow,
    pub memory_bytes: ProfilingMemoryBytes,
    pub utilization: ProfilingUtilization,
    pub cache: ProfilingCache,
    pub fps_instant: f64,
    pub gpu_supported: bool,
    pub window_fps: Vec<WindowFps>,
    pub detail: ProfilingDetailLevel,
    pub frame_report: Option<crate::core::realm::FrameReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowFps {
    pub window_id: u32,
    pub fps_instant: f64,
    pub frame_delta_us: f64,
}

fn to_us(value: u64) -> f64 {
    value as f64 / 1000.0
}

fn rolling_from_samples(
    samples: &std::collections::VecDeque<FrameProfilingSample>,
) -> ProfilingRollingWindow {
    if samples.is_empty() {
        return ProfilingRollingWindow {
            sample_count: 0,
            command_us_avg: 0.0,
            input_us_avg: 0.0,
            render_us_avg: 0.0,
            gpu_us_avg: 0.0,
            fps_avg: 0.0,
            frame_us_p50: 0.0,
            frame_us_p95: 0.0,
            frame_us_p99: 0.0,
            frame_us_max: 0.0,
            render_us_p95: 0.0,
            gpu_us_p95: 0.0,
        };
    }
    let mut command_ns = 0u64;
    let mut input_ns = 0u64;
    let mut render_ns = 0u64;
    let mut gpu_ns = 0u64;
    let mut frame_delta_ns = 0u64;
    let mut frame_ns_samples = Vec::with_capacity(samples.len());
    let mut render_ns_samples = Vec::with_capacity(samples.len());
    let mut gpu_ns_samples = Vec::with_capacity(samples.len());
    for sample in samples {
        command_ns = command_ns.saturating_add(sample.command_ns);
        input_ns = input_ns.saturating_add(sample.input_ns);
        render_ns = render_ns.saturating_add(sample.render_ns);
        gpu_ns = gpu_ns.saturating_add(sample.gpu_ns);
        frame_delta_ns = frame_delta_ns.saturating_add(sample.frame_delta_ns);
        frame_ns_samples.push(sample.frame_delta_ns);
        render_ns_samples.push(sample.render_ns);
        gpu_ns_samples.push(sample.gpu_ns);
    }
    let count = samples.len() as f64;
    let delta_avg_ns = frame_delta_ns as f64 / count;
    ProfilingRollingWindow {
        sample_count: samples.len(),
        command_us_avg: to_us((command_ns as f64 / count) as u64),
        input_us_avg: to_us((input_ns as f64 / count) as u64),
        render_us_avg: to_us((render_ns as f64 / count) as u64),
        gpu_us_avg: to_us((gpu_ns as f64 / count) as u64),
        fps_avg: if delta_avg_ns > 0.0 {
            1_000_000_000.0 / delta_avg_ns
        } else {
            0.0
        },
        frame_us_p50: percentile_us(&mut frame_ns_samples, 0.50),
        frame_us_p95: percentile_us(&mut frame_ns_samples, 0.95),
        frame_us_p99: percentile_us(&mut frame_ns_samples, 0.99),
        frame_us_max: frame_ns_samples
            .iter()
            .copied()
            .max()
            .map(to_us)
            .unwrap_or(0.0),
        render_us_p95: percentile_us(&mut render_ns_samples, 0.95),
        gpu_us_p95: percentile_us(&mut gpu_ns_samples, 0.95),
    }
}

fn percentile_us(values: &mut [u64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_unstable();
    let last_index = values.len().saturating_sub(1);
    let index = ((last_index as f64) * percentile.clamp(0.0, 1.0)).round() as usize;
    to_us(values[index.min(last_index)])
}

fn domain_from_tick(profiling: &TickProfiling) -> ProfilingDomainUs {
    ProfilingDomainUs {
        command: to_us(profiling.command.processing_ns),
        input: to_us(
            profiling
                .input
                .gamepad_processing_ns
                .saturating_add(profiling.input.event_loop_pump_ns),
        ),
        routing: 0.0,
        render: to_us(profiling.render.total_ns),
        gpu: to_us(profiling.gpu.total_ns),
        graph: to_us(
            profiling
                .graph
                .realm_graph_ns
                .saturating_add(profiling.graph.target_graph_ns),
        ),
    }
}

/// Get detailed profiling data from the last tick
pub fn galfus_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> GalfusResult {
    match with_engine(|engine| {
        let mut window_fps = Vec::with_capacity(engine.window.states.len());
        for (&window_id, window_state) in &engine.window.states {
            window_fps.push(WindowFps {
                window_id,
                fps_instant: window_state.fps_instant,
                frame_delta_us: to_us(window_state.last_frame_delta_ns),
            });
        }
        let detail = engine.profiling.config.detail;
        let data = ProfilingData {
            times_us: ProfilingTimesUs {
                command_processing: to_us(engine.profiling.command.processing_ns),
                gamepad_processing: to_us(engine.profiling.input.gamepad_processing_ns),
                event_loop_pump: to_us(engine.profiling.input.event_loop_pump_ns),
                request_redraw: to_us(engine.profiling.render.request_redraw_ns),
                serialization: to_us(engine.profiling.render.serialization_ns),
                render_total: to_us(engine.profiling.render.total_ns),
                render_shadow: to_us(engine.profiling.render.shadow_ns),
                render_windows: to_us(engine.profiling.render.windows_ns),
                frame_delta: to_us(engine.profiling.render.frame_delta_ns),
            },
            domain_us: domain_from_tick(&engine.profiling),
            counters: ProfilingCounters {
                total_events_dispatched: engine.profiling.input.total_events_dispatched,
                total_events_cached: engine.profiling.input.total_events_cached,
            },
            rolling: rolling_from_samples(&engine.profiling.rolling_samples),
            memory_bytes: ProfilingMemoryBytes {
                ram_current: engine.profiling.memory.ram_bytes_current,
                ram_peak: engine.profiling.memory.ram_bytes_peak,
                gpu_current: engine.profiling.memory.gpu_bytes_current,
                gpu_peak: engine.profiling.memory.gpu_bytes_peak,
            },
            utilization: ProfilingUtilization {
                cpu_percent: engine.profiling.utilization.cpu_percent,
                gpu_frame_percent: engine.profiling.utilization.gpu_frame_percent,
                command_percent: engine.profiling.utilization.command_percent,
                input_percent: engine.profiling.utilization.input_percent,
                render_percent: engine.profiling.utilization.render_percent,
                graph_percent: engine.profiling.utilization.graph_percent,
            },
            cache: ProfilingCache {
                render_pipeline_hits: engine.profiling.render.render_pipeline_cache_hits,
                render_pipeline_misses: engine.profiling.render.render_pipeline_cache_misses,
                render_pipeline_evictions: engine.profiling.render.render_pipeline_cache_evictions,
                compute_pipeline_hits: engine.profiling.render.compute_pipeline_cache_hits,
                compute_pipeline_misses: engine.profiling.render.compute_pipeline_cache_misses,
                compute_pipeline_evictions: engine
                    .profiling
                    .render
                    .compute_pipeline_cache_evictions,
                compose_bind_cache_hits: engine.profiling.render.compose_bind_cache_hits,
                compose_bind_cache_misses: engine.profiling.render.compose_bind_cache_misses,
                compose_bind_cache_evictions: engine.profiling.render.compose_bind_cache_evictions,
                post_bind_cache_hits: engine.profiling.render.post_bind_cache_hits,
                post_bind_cache_misses: engine.profiling.render.post_bind_cache_misses,
                post_bind_cache_evictions: engine.profiling.render.post_bind_cache_evictions,
                material_shader_module_evictions: engine
                    .profiling
                    .render
                    .material_shader_module_evictions,
                material_program_cache_evictions: engine
                    .profiling
                    .render
                    .material_program_cache_evictions,
            },
            fps_instant: if engine.profiling.render.frame_delta_ns > 0 {
                1_000_000_000.0 / engine.profiling.render.frame_delta_ns as f64
            } else {
                0.0
            },
            gpu_supported: engine.gpu_profiler.is_some(),
            window_fps,
            detail,
            frame_report: if detail == ProfilingDetailLevel::Full {
                Some(engine.universal_state.composition.frame_report.clone())
            } else {
                None
            },
        };

        let serialized_data = match rmp_serde::to_vec_named(&data) {
            Ok(data) => data,
            Err(_) => return GalfusResult::UnknownError,
        };

        let data_length = serialized_data.len();
        let boxed = serialized_data.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u8;

        unsafe {
            *out_ptr = ptr;
            *out_length = data_length;
        }

        GalfusResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
