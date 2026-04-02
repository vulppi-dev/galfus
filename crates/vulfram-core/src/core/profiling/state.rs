use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfilingDetailLevel {
    Basic,
    Full,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingConfig {
    pub enabled: bool,
    pub detail: ProfilingDetailLevel,
    pub sampling_percent: u8,
    pub window_frames: u8,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detail: ProfilingDetailLevel::Full,
            sampling_percent: 100,
            window_frames: 30,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandProfiling {
    pub processing_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct InputProfiling {
    pub gamepad_processing_ns: u64,
    pub event_loop_pump_ns: u64,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
    pub custom_events_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RenderProfiling {
    pub request_redraw_ns: u64,
    pub serialization_ns: u64,
    pub total_ns: u64,
    pub shadow_ns: u64,
    pub windows_ns: u64,
    pub frame_delta_ns: u64,
    pub render_pipeline_cache_hits: u32,
    pub render_pipeline_cache_misses: u32,
    pub compute_pipeline_cache_hits: u32,
    pub compute_pipeline_cache_misses: u32,
    pub compose_bind_cache_hits: u32,
    pub compose_bind_cache_misses: u32,
    pub post_bind_cache_hits: u32,
    pub post_bind_cache_misses: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GpuProfiling {
    pub shadow_ns: u64,
    pub light_cull_ns: u64,
    pub forward_ns: u64,
    pub compose_ns: u64,
    pub total_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct UiProfiling {
    pub input_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GraphProfiling {
    pub realm_graph_ns: u64,
    pub target_graph_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryProfiling {
    pub ram_bytes_current: u64,
    pub ram_bytes_peak: u64,
    pub gpu_bytes_current: u64,
    pub gpu_bytes_peak: u64,
}

#[derive(Debug, Clone, Default)]
pub struct UtilizationProfiling {
    pub cpu_percent: f32,
    pub gpu_frame_percent: f32,
    pub command_percent: f32,
    pub input_percent: f32,
    pub render_percent: f32,
    pub ui_percent: f32,
    pub graph_percent: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameProfilingSample {
    pub command_ns: u64,
    pub input_ns: u64,
    pub render_ns: u64,
    pub gpu_ns: u64,
    pub frame_delta_ns: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TickProfiling {
    pub command: CommandProfiling,
    pub input: InputProfiling,
    pub render: RenderProfiling,
    pub gpu: GpuProfiling,
    pub ui: UiProfiling,
    pub graph: GraphProfiling,
    pub memory: MemoryProfiling,
    pub utilization: UtilizationProfiling,
    pub config: ProfilingConfig,
    pub rolling_samples: VecDeque<FrameProfilingSample>,
    pub sampled_this_frame: bool,
    pub(crate) process_cpu_ticks: Option<u64>,
    pub(crate) system_cpu_ticks: Option<u64>,
}

impl TickProfiling {
    pub fn begin_frame(&mut self, delta_time_ms: u32, frame_index: u64) {
        self.command.processing_ns = 0;
        self.input.gamepad_processing_ns = 0;
        self.input.event_loop_pump_ns = 0;
        self.input.total_events_dispatched = 0;
        self.input.total_events_cached = 0;
        self.input.custom_events_ns = 0;
        self.render.request_redraw_ns = 0;
        self.render.serialization_ns = 0;
        self.render.total_ns = 0;
        self.render.shadow_ns = 0;
        self.render.windows_ns = 0;
        self.render.frame_delta_ns = (delta_time_ms as u64).saturating_mul(1_000_000);
        self.render.render_pipeline_cache_hits = 0;
        self.render.render_pipeline_cache_misses = 0;
        self.render.compute_pipeline_cache_hits = 0;
        self.render.compute_pipeline_cache_misses = 0;
        self.render.compose_bind_cache_hits = 0;
        self.render.compose_bind_cache_misses = 0;
        self.render.post_bind_cache_hits = 0;
        self.render.post_bind_cache_misses = 0;
        self.gpu.shadow_ns = 0;
        self.gpu.light_cull_ns = 0;
        self.gpu.forward_ns = 0;
        self.gpu.compose_ns = 0;
        self.gpu.total_ns = 0;
        self.ui.input_ns = 0;
        self.graph.realm_graph_ns = 0;
        self.graph.target_graph_ns = 0;
        self.utilization.gpu_frame_percent = 0.0;
        self.utilization.command_percent = 0.0;
        self.utilization.input_percent = 0.0;
        self.utilization.render_percent = 0.0;
        self.utilization.ui_percent = 0.0;
        self.utilization.graph_percent = 0.0;
        self.sampled_this_frame = self.should_sample(frame_index);
    }

    pub fn should_sample(&self, frame_index: u64) -> bool {
        if !self.config.enabled {
            return false;
        }
        let sampling = self.config.sampling_percent.min(100);
        if sampling == 0 {
            return false;
        }
        if sampling >= 100 {
            return true;
        }
        frame_index % 100 < sampling as u64
    }

    pub fn push_rolling_sample(&mut self) {
        if !self.sampled_this_frame {
            return;
        }
        let sample = FrameProfilingSample {
            command_ns: self.command.processing_ns,
            input_ns: self.input.gamepad_processing_ns + self.input.event_loop_pump_ns,
            render_ns: self.render.total_ns,
            gpu_ns: self.gpu.total_ns,
            frame_delta_ns: self.render.frame_delta_ns,
        };
        self.rolling_samples.push_back(sample);
        let max_frames = self.config.window_frames.max(1) as usize;
        while self.rolling_samples.len() > max_frames {
            self.rolling_samples.pop_front();
        }
    }

    pub fn update_budget_utilization(&mut self) {
        let frame_delta_ns = self.render.frame_delta_ns.max(1) as f32;
        self.utilization.command_percent =
            (self.command.processing_ns as f32 / frame_delta_ns) * 100.0;
        self.utilization.input_percent = ((self.input.gamepad_processing_ns
            + self.input.event_loop_pump_ns) as f32
            / frame_delta_ns)
            * 100.0;
        self.utilization.render_percent = (self.render.total_ns as f32 / frame_delta_ns) * 100.0;
        self.utilization.ui_percent = (self.ui.input_ns as f32 / frame_delta_ns) * 100.0;
        self.utilization.graph_percent =
            ((self.graph.realm_graph_ns + self.graph.target_graph_ns) as f32 / frame_delta_ns)
                * 100.0;
        self.utilization.gpu_frame_percent = (self.gpu.total_ns as f32 / frame_delta_ns) * 100.0;
    }
}
