pub mod cache;
pub mod gizmos;
pub mod graph;
mod passes;
mod realm_graph;
pub mod state;

use crate::core::realm::{FrameReport, RealmGraphPlanner};
use crate::core::render::graph::RenderGraphPlan;
use crate::core::state::EngineState;
use realm_graph::{
    collect_connectors_by_realm, collect_cut_connectors, collect_present_sizes,
    collect_surface_views, compose_realm_connectors, ensure_surface_target, map_realms_to_windows,
    resolve_realm_surface, update_surface_cache,
};
pub use state::RenderState;
use std::collections::HashSet;

pub fn bloom_chain_size(base: u32, level: usize) -> u32 {
    passes::bloom_chain_size(base, level)
}

#[cfg(feature = "wasm")]
use js_sys::Date;

#[cfg(feature = "wasm")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

pub fn render_frames(engine_state: &mut EngineState) {
    engine_state.profiling.render_total_ns = 0;
    engine_state.profiling.render_shadow_ns = 0;
    engine_state.profiling.render_windows_ns = 0;
    engine_state.profiling.gpu_shadow_ns = 0;
    engine_state.profiling.gpu_light_cull_ns = 0;
    engine_state.profiling.gpu_forward_ns = 0;
    engine_state.profiling.gpu_compose_ns = 0;
    engine_state.profiling.gpu_total_ns = 0;

    let (target_plan, target_diff) = {
        let cache = &mut engine_state.universal_state.target_graph_cache;
        let diff = cache
            .update(
                &engine_state.universal_state.targets.entries,
                &engine_state.universal_state.target_binds.entries,
                &engine_state.universal_state.realms,
            )
            .cloned();
        let plan = cache.last_plan.clone();
        (plan, diff)
    };
    crate::core::target::sync_auto_graph(engine_state);

    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    if let Some(gpu_profiler) = engine_state.gpu_profiler.as_mut() {
        gpu_profiler.ensure_capacity(device, queue, engine_state.window.states.len());
    }

    let time = engine_state.time as f32 / 1000.0;
    let delta_time = engine_state.delta_time as f32 / 1000.0;
    let frame_index = engine_state.frame_index as u32;
    let frame_spec = crate::core::resources::FrameComponent::new(time, delta_time, frame_index);
    let mut gpu_written = false;

    #[cfg(not(feature = "wasm"))]
    let total_start = std::time::Instant::now();
    #[cfg(feature = "wasm")]
    let total_start = now_ns();

    // 1. Update Shadows (Global for all realms with a shadow pass)
    let shadow_enabled = engine_state
        .universal_state
        .realms
        .entries
        .values()
        .filter_map(|entry| entry.value.render_graph.as_ref())
        .any(|graph| graph.plan().has_pass("shadow"));

    if shadow_enabled {
        if let Some((_, window_state)) = engine_state.window.states.iter_mut().next() {
            #[cfg(not(feature = "wasm"))]
            let shadow_start = std::time::Instant::now();
            #[cfg(feature = "wasm")]
            let shadow_start = now_ns();
            // Ensure data is ready but WITHOUT shadow atlas binding to avoid conflicts
            window_state
                .render_state
                .prepare_render(device, frame_spec, false);

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Update Encoder"),
            });

            if let Some(gpu_profiler) = engine_state.gpu_profiler.as_ref() {
                if gpu_profiler.query_count() >= 2 {
                    encoder.write_timestamp(gpu_profiler.query_set(), 0);
                    gpu_written = true;
                }
            }

            passes::pass_shadow_update(
                &mut window_state.render_state,
                device,
                queue,
                &mut encoder,
                engine_state.frame_index,
            );

            if let Some(gpu_profiler) = engine_state.gpu_profiler.as_ref() {
                if gpu_profiler.query_count() >= 2 {
                    encoder.write_timestamp(gpu_profiler.query_set(), 1);
                    gpu_written = true;
                }
            }

            if let Some(shadow) = &mut window_state.render_state.shadow {
                shadow.sync_table();
            }

            queue.submit(Some(encoder.finish()));
            #[cfg(not(feature = "wasm"))]
            {
                engine_state.profiling.render_shadow_ns = shadow_start.elapsed().as_nanos() as u64;
            }
            #[cfg(feature = "wasm")]
            {
                engine_state.profiling.render_shadow_ns = now_ns().saturating_sub(shadow_start);
            }
        }
    }

    // 2. Render all realms (RealmGraph order)
    let mut windows_ns: u64 = 0;
    let realm_plan = RealmGraphPlanner::default().build_plan(&engine_state.universal_state);
    let cut_connectors = collect_cut_connectors(&realm_plan);
    update_surface_cache(&mut engine_state.universal_state, &cut_connectors);
    let mut frame_report =
        FrameReport::from_plan(&realm_plan, &engine_state.universal_state.surface_cache);
    frame_report.apply_target_graph_stats(&target_plan, target_diff.as_ref());
    let realm_windows = map_realms_to_windows(&engine_state.universal_state);
    collect_present_sizes(
        &engine_state.universal_state,
        &engine_state.window.states,
        &mut engine_state.present_sizes_cache,
        &mut engine_state.present_sizes_hash,
    );
    let present_sizes = &engine_state.present_sizes_cache;
    let connectors_by_realm = collect_connectors_by_realm(&engine_state.universal_state);
    let surface_views = collect_surface_views(
        device,
        &engine_state.universal_state,
        &mut engine_state.surface_targets,
        present_sizes,
    );
    let mut updated_surfaces: HashSet<crate::core::realm::SurfaceId> = HashSet::new();
    const MAX_REALM_ITERATIONS: u32 = 1;
    let mut iteration: u32 = 0;
    loop {
        frame_report.no_progress_realms.clear();
        let mut window_counter: u32 = 0;

        for realm_id in &realm_plan.order {
            let Some(window_id) = realm_windows.get(realm_id) else {
                continue;
            };
            let Some(window_state) = engine_state.window.states.get_mut(window_id) else {
                continue;
            };
            let Some(surface_id) = resolve_realm_surface(&engine_state.universal_state, *realm_id)
            else {
                continue;
            };
            let Some(realm_entry) = engine_state
                .universal_state
                .realms
                .entries
                .get_mut(realm_id)
            else {
                continue;
            };
            if !should_render_realm(realm_entry, engine_state.frame_index) {
                FrameReport::push_unique(&mut frame_report.throttled_realms, realm_id.0);
                continue;
            }

            let target_size = engine_state
                .universal_state
                .surfaces
                .entries
                .get(&surface_id)
                .map(|entry| entry.value.size)
                .unwrap_or(window_state.inner_size);
            let target_format = engine_state
                .universal_state
                .surfaces
                .entries
                .get(&surface_id)
                .and_then(|entry| entry.value.format_policy)
                .unwrap_or(wgpu::TextureFormat::Rgba16Float);

            let surface_target = ensure_surface_target(
                device,
                &mut engine_state.surface_targets,
                surface_id,
                target_size,
                target_format,
            );

            #[cfg(not(feature = "wasm"))]
            let window_start = std::time::Instant::now();
            #[cfg(feature = "wasm")]
            let window_start = now_ns();

            let render_state = &mut window_state.render_state;
            render_state.prepare_render(device, frame_spec, true);

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Realm Target Clear"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_target.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }

            let gpu_base = engine_state.gpu_profiler.as_ref().and_then(|gpu_profiler| {
                let base = 2 + (window_counter as u32) * 6;
                if gpu_profiler.query_count() >= base + 6 {
                    Some(base)
                } else {
                    None
                }
            });
            window_counter = window_counter.saturating_add(1);

            let plan = match realm_entry.value.render_graph.as_ref() {
                Some(graph) => graph.plan(),
                None => {
                    log::error!("Realm {} is missing a render graph", realm_id.0);
                    FrameReport::push_unique(&mut frame_report.no_progress_realms, realm_id.0);
                    continue;
                }
            };
            gpu_written |= execute_graph_to_view(
                &plan,
                render_state,
                device,
                queue,
                &mut encoder,
                &surface_target.view,
                surface_target.format,
                target_size,
                engine_state.frame_index,
                engine_state.gpu_profiler.as_ref(),
                gpu_base,
            );

            compose_realm_connectors(
                render_state,
                device,
                &mut encoder,
                &engine_state.universal_state,
                &connectors_by_realm,
                *realm_id,
                surface_id,
                &cut_connectors,
                &surface_views,
                &surface_target.view,
                surface_target.format,
                target_size,
                engine_state.frame_index,
                &mut frame_report,
            );

            updated_surfaces.insert(surface_id);

            queue.submit(Some(encoder.finish()));
            #[cfg(not(feature = "wasm"))]
            {
                windows_ns = windows_ns.saturating_add(window_start.elapsed().as_nanos() as u64);
            }
            #[cfg(feature = "wasm")]
            {
                windows_ns = windows_ns.saturating_add(now_ns().saturating_sub(window_start));
            }
        }

        for present in engine_state.universal_state.presents.entries.values() {
            if !updated_surfaces.contains(&present.value.surface) {
                continue;
            }
            let window_id = present.value.window_id;
            let Some(window_state) = engine_state.window.states.get_mut(&window_id) else {
                continue;
            };
            let Some(surface_target) = engine_state.surface_targets.get(&present.value.surface)
            else {
                continue;
            };

            let surface_texture = match window_state.surface.get_current_texture() {
                Ok(texture) => texture,
                Err(e) => {
                    log::error!("Failed to get surface texture: {:?}", e);
                    continue;
                }
            };

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            passes::pass_compose_surface(
                &mut window_state.render_state,
                device,
                &mut encoder,
                &surface_view,
                window_state.config.format,
                glam::UVec2::new(window_state.config.width, window_state.config.height),
                &surface_target.view,
                engine_state.frame_index,
            );

            queue.submit(Some(encoder.finish()));
            surface_texture.present();
            #[cfg(not(feature = "wasm"))]
            {
                let now = std::time::Instant::now();
                let delta_ns = window_state
                    .last_present_instant
                    .map(|prev| now.duration_since(prev).as_nanos() as u64)
                    .unwrap_or(0);
                window_state.last_present_instant = Some(now);
                window_state.last_frame_delta_ns = delta_ns;
                window_state.fps_instant = if delta_ns > 0 {
                    1_000_000_000.0 / delta_ns as f64
                } else {
                    0.0
                };
            }
            #[cfg(feature = "wasm")]
            {
                let now = now_ns();
                let delta_ns = if window_state.last_present_ns > 0 {
                    now.saturating_sub(window_state.last_present_ns)
                } else {
                    0
                };
                window_state.last_present_ns = now;
                window_state.last_frame_delta_ns = delta_ns;
                window_state.fps_instant = if delta_ns > 0 {
                    1_000_000_000.0 / delta_ns as f64
                } else {
                    0.0
                };
            }
        }

        iteration = iteration.saturating_add(1);
        if frame_report.no_progress_realms.is_empty() || iteration >= MAX_REALM_ITERATIONS {
            break;
        }
    }

    engine_state.universal_state.frame_report = frame_report;

    if gpu_written {
        if let Some(gpu_profiler) = engine_state.gpu_profiler.as_mut() {
            if gpu_profiler.query_count() > 0 {
                let mut resolve_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("GpuProfiler Resolve Encoder"),
                    });
                resolve_encoder.resolve_query_set(
                    gpu_profiler.query_set(),
                    0..gpu_profiler.query_count(),
                    gpu_profiler.resolve_buffer(),
                    0,
                );
                resolve_encoder.copy_buffer_to_buffer(
                    gpu_profiler.resolve_buffer(),
                    0,
                    gpu_profiler.readback_buffer(),
                    0,
                    gpu_profiler.buffer_size(),
                );
                queue.submit(Some(resolve_encoder.finish()));
                gpu_profiler.readback_and_update(device, &mut engine_state.profiling);
            }
        }
    }
    engine_state.profiling.render_windows_ns = windows_ns;
    #[cfg(not(feature = "wasm"))]
    {
        engine_state.profiling.render_total_ns = total_start.elapsed().as_nanos() as u64;
    }
    #[cfg(feature = "wasm")]
    {
        engine_state.profiling.render_total_ns = now_ns().saturating_sub(total_start);
    }
}

fn execute_graph_to_view(
    plan: &RenderGraphPlan,
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    gpu_base: Option<u32>,
) -> bool {
    let mut gpu_written = false;
    let mut skybox_done = false;

    for &node_idx in &plan.order {
        let node = &plan.nodes[node_idx];
        match node.pass_id.as_str() {
            "shadow" => {
                continue;
            }
            "skybox" => {
                skybox_done = passes::pass_skybox(render_state, device, queue, encoder, frame_index);
            }
            "light-cull" => {
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base, &mut gpu_written);
                }
                passes::pass_light_cull(render_state, device, encoder, frame_index);
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 1, &mut gpu_written);
                }
            }
            "forward" => {
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 2, &mut gpu_written);
                }
                passes::pass_forward(
                    render_state,
                    device,
                    queue,
                    encoder,
                    frame_index,
                    !skybox_done,
                );
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 3, &mut gpu_written);
                }
            }
            "outline" => {
                passes::pass_outline(render_state, device, queue, encoder, frame_index);
            }
            "ssao" => {
                passes::pass_ssao(render_state, device, queue, encoder, frame_index);
            }
            "ssao-blur" => {
                passes::pass_ssao_blur(render_state, device, queue, encoder, frame_index);
            }
            "bloom" => {
                passes::pass_bloom(render_state, device, queue, encoder, frame_index);
            }
            "post" => {
                passes::pass_post(render_state, device, queue, encoder, frame_index);
            }
            "compose" => {
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 4, &mut gpu_written);
                }
                passes::pass_compose_to_view(
                    render_state,
                    device,
                    queue,
                    encoder,
                    target_view,
                    target_format,
                    target_size.x,
                    target_size.y,
                    frame_index,
                );
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 5, &mut gpu_written);
                }
            }
            _ => {}
        }
    }

    gpu_written
}

fn should_render_realm(
    entry: &mut crate::core::realm::TableEntry<crate::core::realm::RealmState>,
    frame_index: u64,
) -> bool {
    let importance = entry.value.importance;
    if importance == 0 {
        return false;
    }
    let base_interval: u64 = match importance {
        1 => 1,
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let cache_multiplier: u64 = match entry.value.cache_policy {
        0 => 1,
        1 => 2,
        2 => 4,
        _ => 1,
    };
    let interval = base_interval.saturating_mul(cache_multiplier);
    let should_render = frame_index.saturating_sub(entry.value.last_render_frame) >= interval;
    if should_render {
        entry.value.last_render_frame = frame_index;
    }
    should_render
}

fn write_gpu_timestamp(
    encoder: &mut wgpu::CommandEncoder,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    index: u32,
    gpu_written: &mut bool,
) {
    if let Some(profiler) = gpu_profiler {
        encoder.write_timestamp(profiler.query_set(), index);
        *gpu_written = true;
    }
}
