use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum ShaderId {
    Compose = 0,
    Post,
    Outline,
    Ssao,
    SsaoBlur,
    SsaoMsaa,
    SsaoBlurMsaa,
    BloomPrefilterH,
    BloomPrefilterV,
    BloomDownsample,
    BloomUpsample,
    BloomCombine,
    Skybox,
    Shadow,
    LightCull,
    ForwardStandard,
    ForwardPbr,
    Gizmo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shader_id: u64,
    pub color_format: wgpu::TextureFormat,
    pub color_target_count: u8,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub sample_count: u32,
    pub topology: wgpu::PrimitiveTopology,
    pub polygon_mode: wgpu::PolygonMode,
    pub cull_mode: Option<wgpu::Face>,
    pub front_face: wgpu::FrontFace,
    pub depth_write_enabled: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub blend: Option<wgpu::BlendState>,
}

#[derive(Debug)]
struct PipelineEntry {
    pipeline: wgpu::RenderPipeline,
    last_used_frame: u64,
}

#[derive(Debug)]
pub struct RenderCache {
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    compute_pipelines: HashMap<ComputePipelineKey, ComputePipelineEntry>,
    max_unused_frames: u64,
    frame_stats: RenderCacheStats,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderCacheStats {
    pub render_pipeline_hits: u32,
    pub render_pipeline_misses: u32,
    pub compute_pipeline_hits: u32,
    pub compute_pipeline_misses: u32,
    pub render_pipeline_evictions: u32,
    pub compute_pipeline_evictions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputePipelineKey {
    pub shader_id: u64,
}

#[derive(Debug)]
struct ComputePipelineEntry {
    pipeline: wgpu::ComputePipeline,
    last_used_frame: u64,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            max_unused_frames: 3,
            frame_stats: RenderCacheStats::default(),
        }
    }

    pub fn get_or_create<F>(
        &mut self,
        key: PipelineKey,
        frame_index: u64,
        create: F,
    ) -> &wgpu::RenderPipeline
    where
        F: FnOnce() -> wgpu::RenderPipeline,
    {
        match self.pipelines.entry(key) {
            Entry::Occupied(entry) => {
                self.frame_stats.render_pipeline_hits =
                    self.frame_stats.render_pipeline_hits.saturating_add(1);
                let entry = entry.into_mut();
                entry.last_used_frame = frame_index;
                &entry.pipeline
            }
            Entry::Vacant(entry) => {
                self.frame_stats.render_pipeline_misses =
                    self.frame_stats.render_pipeline_misses.saturating_add(1);
                let entry = entry.insert(PipelineEntry {
                    pipeline: create(),
                    last_used_frame: frame_index,
                });
                &entry.pipeline
            }
        }
    }

    pub fn gc(&mut self, frame_index: u64) {
        let max_unused = self.max_unused_frames;
        let before_render = self.pipelines.len();
        self.pipelines
            .retain(|_, entry| frame_index.saturating_sub(entry.last_used_frame) <= max_unused);
        let evicted_render = before_render.saturating_sub(self.pipelines.len());
        self.frame_stats.render_pipeline_evictions = self
            .frame_stats
            .render_pipeline_evictions
            .saturating_add(evicted_render as u32);
        let before_compute = self.compute_pipelines.len();
        self.compute_pipelines
            .retain(|_, entry| frame_index.saturating_sub(entry.last_used_frame) <= max_unused);
        let evicted_compute = before_compute.saturating_sub(self.compute_pipelines.len());
        self.frame_stats.compute_pipeline_evictions = self
            .frame_stats
            .compute_pipeline_evictions
            .saturating_add(evicted_compute as u32);
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.compute_pipelines.clear();
        self.frame_stats = RenderCacheStats::default();
    }

    pub fn get_or_create_compute<F>(
        &mut self,
        key: ComputePipelineKey,
        frame_index: u64,
        create: F,
    ) -> &wgpu::ComputePipeline
    where
        F: FnOnce() -> wgpu::ComputePipeline,
    {
        match self.compute_pipelines.entry(key) {
            Entry::Occupied(entry) => {
                self.frame_stats.compute_pipeline_hits =
                    self.frame_stats.compute_pipeline_hits.saturating_add(1);
                let entry = entry.into_mut();
                entry.last_used_frame = frame_index;
                &entry.pipeline
            }
            Entry::Vacant(entry) => {
                self.frame_stats.compute_pipeline_misses =
                    self.frame_stats.compute_pipeline_misses.saturating_add(1);
                let entry = entry.insert(ComputePipelineEntry {
                    pipeline: create(),
                    last_used_frame: frame_index,
                });
                &entry.pipeline
            }
        }
    }

    pub fn frame_stats(&self) -> RenderCacheStats {
        self.frame_stats
    }

    pub fn reset_frame_stats(&mut self) {
        self.frame_stats = RenderCacheStats::default();
    }
}
