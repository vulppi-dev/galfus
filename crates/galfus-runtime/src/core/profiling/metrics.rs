use crate::core::state::EngineState;

pub fn refresh_runtime_metrics(engine_state: &mut EngineState) {
    super::process::refresh_process_metrics(&mut engine_state.profiling);
    refresh_render_cache_metrics(engine_state);

    let gpu_bytes_current = estimate_engine_gpu_bytes(engine_state);
    engine_state.profiling.memory.gpu_bytes_current = gpu_bytes_current;
    engine_state.profiling.memory.gpu_bytes_peak = engine_state
        .profiling
        .memory
        .gpu_bytes_peak
        .max(gpu_bytes_current);
    engine_state.profiling.update_budget_utilization();
}

fn refresh_render_cache_metrics(engine_state: &mut EngineState) {
    let mut render_pipeline_hits = 0u32;
    let mut render_pipeline_misses = 0u32;
    let mut render_pipeline_evictions = 0u32;
    let mut compute_pipeline_hits = 0u32;
    let mut compute_pipeline_misses = 0u32;
    let mut compute_pipeline_evictions = 0u32;
    let mut compose_bind_cache_hits = 0u32;
    let mut compose_bind_cache_misses = 0u32;
    let mut compose_bind_cache_evictions = 0u32;
    let mut post_bind_cache_hits = 0u32;
    let mut post_bind_cache_misses = 0u32;
    let mut post_bind_cache_evictions = 0u32;
    let mut material_shader_module_evictions = 0u32;

    for render_state in engine_state.render.states.values() {
        let cache_stats = render_state.cache.frame_stats();
        render_pipeline_hits =
            render_pipeline_hits.saturating_add(cache_stats.render_pipeline_hits);
        render_pipeline_misses =
            render_pipeline_misses.saturating_add(cache_stats.render_pipeline_misses);
        render_pipeline_evictions =
            render_pipeline_evictions.saturating_add(cache_stats.render_pipeline_evictions);
        compute_pipeline_hits =
            compute_pipeline_hits.saturating_add(cache_stats.compute_pipeline_hits);
        compute_pipeline_misses =
            compute_pipeline_misses.saturating_add(cache_stats.compute_pipeline_misses);
        compute_pipeline_evictions =
            compute_pipeline_evictions.saturating_add(cache_stats.compute_pipeline_evictions);
        compose_bind_cache_hits =
            compose_bind_cache_hits.saturating_add(render_state.compose_bind_cache_hits);
        compose_bind_cache_misses =
            compose_bind_cache_misses.saturating_add(render_state.compose_bind_cache_misses);
        compose_bind_cache_evictions =
            compose_bind_cache_evictions.saturating_add(render_state.compose_bind_cache_evictions);
        post_bind_cache_hits =
            post_bind_cache_hits.saturating_add(render_state.post_bind_cache_hits);
        post_bind_cache_misses =
            post_bind_cache_misses.saturating_add(render_state.post_bind_cache_misses);
        post_bind_cache_evictions =
            post_bind_cache_evictions.saturating_add(render_state.post_bind_cache_evictions);
        material_shader_module_evictions = material_shader_module_evictions
            .saturating_add(render_state.material_shader_module_evictions);
    }

    engine_state.profiling.render.render_pipeline_cache_hits = render_pipeline_hits;
    engine_state.profiling.render.render_pipeline_cache_misses = render_pipeline_misses;
    engine_state
        .profiling
        .render
        .render_pipeline_cache_evictions = render_pipeline_evictions;
    engine_state.profiling.render.compute_pipeline_cache_hits = compute_pipeline_hits;
    engine_state.profiling.render.compute_pipeline_cache_misses = compute_pipeline_misses;
    engine_state
        .profiling
        .render
        .compute_pipeline_cache_evictions = compute_pipeline_evictions;
    engine_state.profiling.render.compose_bind_cache_hits = compose_bind_cache_hits;
    engine_state.profiling.render.compose_bind_cache_misses = compose_bind_cache_misses;
    engine_state.profiling.render.compose_bind_cache_evictions = compose_bind_cache_evictions;
    engine_state.profiling.render.post_bind_cache_hits = post_bind_cache_hits;
    engine_state.profiling.render.post_bind_cache_misses = post_bind_cache_misses;
    engine_state.profiling.render.post_bind_cache_evictions = post_bind_cache_evictions;
    engine_state
        .profiling
        .render
        .material_shader_module_evictions = material_shader_module_evictions;
}

fn estimate_engine_gpu_bytes(engine_state: &EngineState) -> u64 {
    let surface_target_bytes = engine_state
        .surface_targets
        .values()
        .map(galfus_render::RenderTarget::estimated_bytes)
        .sum::<u64>();
    let render_state_bytes = engine_state
        .render
        .states
        .values()
        .map(crate::core::render::RenderState::estimated_gpu_bytes)
        .sum::<u64>();
    let gpu_profiler_bytes = engine_state
        .gpu_profiler
        .as_ref()
        .map(|gpu_profiler| gpu_profiler.buffer_size().saturating_mul(2))
        .unwrap_or(0);

    surface_target_bytes
        .saturating_add(render_state_bytes)
        .saturating_add(gpu_profiler_bytes)
}
