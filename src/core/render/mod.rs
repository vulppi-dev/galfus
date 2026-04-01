pub mod cache;
mod frame_helpers;
pub mod gizmos;
pub mod graph;
mod graph_execute;
mod passes;
mod realm_graph;
pub mod runtime;
mod scene_sync;
pub mod state;
mod ui_platform_actions;
use crate::core::realm::{FrameReport, RealmGraphPlanner, apply_target_graph_stats};
use crate::core::render::passes::UiPlatformAction;
use crate::core::state::EngineState;
use crate::core::system::push_error_event;
use crate::core::ui::events::UiEvent;
use frame_helpers::{
    apply_realm_environment_bindings, apply_target_size_requests, build_soft_cut_diagnostic,
    build_target_surface_map, collect_window_camera_target_sizes, refresh_window_target_textures,
    should_render_realm,
};
use graph_execute::execute_graph_to_view;
use realm_graph::{
    collect_connectors_by_realm, collect_cut_connectors, collect_present_sizes,
    collect_surface_views, compose_realm_connectors, ensure_surface_target, map_realms_to_windows,
    resolve_realm_surface, update_surface_cache,
};
pub use runtime::RenderManager;
use scene_sync::{sync_scene_from_realm_and_universal_resources, sync_window_geometry_registry};
pub use state::RenderState;
use std::collections::HashSet;
use ui_platform_actions::apply_ui_platform_actions;
pub fn bloom_chain_size(base: u32, level: usize) -> u32 {
    passes::bloom_chain_size(base, level)
}

#[cfg(feature = "wasm")]
use js_sys::Date;

#[cfg(feature = "wasm")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

fn resolve_realm_render_graph<'a>(
    universal: &'a crate::core::realm::UniversalState,
    realm_id: crate::core::realm::RealmId,
) -> Option<&'a crate::core::render::graph::RenderGraphState> {
    let realm = universal.realms.entries.get(&realm_id)?;
    let render_graph_id =
        vulfram_render::resolve_render_graph_id(realm.value.render_graph_id, realm.value.kind);
    if let Some(graph) = universal.render_graphs.get(&render_graph_id) {
        return Some(&graph.state);
    }
    None
}

pub fn render_frames(engine_state: &mut EngineState) {
    engine_state.profiling.render.total_ns = 0;
    engine_state.profiling.render.shadow_ns = 0;
    engine_state.profiling.render.windows_ns = 0;
    engine_state.profiling.gpu.shadow_ns = 0;
    engine_state.profiling.gpu.light_cull_ns = 0;
    engine_state.profiling.gpu.forward_ns = 0;
    engine_state.profiling.gpu.compose_ns = 0;
    engine_state.profiling.gpu.total_ns = 0;

    let target_size_requests =
        std::mem::take(&mut engine_state.universal_state.ui.target_size_requests);
    apply_target_size_requests(engine_state, &target_size_requests);

    let (target_plan, target_diff) = {
        let cache = &mut engine_state.universal_state.target_graph_cache;
        let diff = cache
            .update(
                &engine_state.universal_state.targets.entries,
                &engine_state.universal_state.target_layers.entries,
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

    let time = engine_state.runtime.frame.time as f32 / 1000.0;
    let delta_time = engine_state.runtime.frame.delta_time as f32 / 1000.0;
    let frame_index = engine_state.runtime.frame.frame_index as u32;
    let frame_spec = crate::core::resources::FrameComponent::new(time, delta_time, frame_index);
    let mut gpu_written = false;

    #[cfg(not(feature = "wasm"))]
    let total_start = std::time::Instant::now();
    #[cfg(feature = "wasm")]
    let total_start = now_ns();

    // 1. Render all realms (RealmGraph order)
    let mut windows_ns: u64 = 0;
    let mut shadow_ns: u64 = 0;
    let realm_plan = RealmGraphPlanner::default().build_plan(&engine_state.universal_state);
    let cut_connectors = collect_cut_connectors(&realm_plan);
    update_surface_cache(&mut engine_state.universal_state, &cut_connectors);
    let previous_cut_edges = engine_state.universal_state.frame_report.cut_edges.len();
    let mut frame_report =
        FrameReport::from_plan(&realm_plan, &engine_state.universal_state.surface_cache);
    apply_target_graph_stats(&mut frame_report, &target_plan, target_diff.as_ref());
    frame_report.target_autolink_failures = engine_state
        .universal_state
        .target_autolink_failures
        .clone();
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
    let target_surface_map = build_target_surface_map(
        &engine_state.universal_state.targets,
        &engine_state.universal_state.auto_links,
    );
    refresh_window_target_textures(
        &mut engine_state.render.states,
        &engine_state
            .universal_state
            .render_resources
            .target_texture_binds,
        &target_surface_map,
        &engine_state.surface_targets,
    );
    let mut updated_surfaces: HashSet<crate::core::realm::SurfaceId> = HashSet::new();
    let mut ui_events: Vec<UiEvent> = Vec::new();
    let mut ui_platform_actions: Vec<UiPlatformAction> = Vec::new();
    let mut synced_windows: HashSet<u32> = HashSet::new();
    const MAX_REALM_ITERATIONS: u32 = 1;
    let mut iteration: u32 = 0;
    loop {
        frame_report.no_progress_realms.clear();
        let mut window_counter: u32 = 0;

        for realm_id in &realm_plan.order {
            let Some(window_id) = realm_windows.get(realm_id) else {
                continue;
            };
            let Some(window_state) = engine_state.window.states.get(window_id) else {
                continue;
            };
            let Some(render_state) = engine_state.render.get_mut(window_id) else {
                continue;
            };
            let Some(surface_id) = resolve_realm_surface(&engine_state.universal_state, *realm_id)
            else {
                continue;
            };
            let should_render = engine_state
                .universal_state
                .realms
                .entries
                .get_mut(realm_id)
                .map(|realm_entry| {
                    should_render_realm(realm_entry, engine_state.runtime.frame.frame_index)
                })
                .unwrap_or(false);
            if !should_render {
                FrameReport::push_unique(&mut frame_report.throttled_realms, realm_id.0);
                continue;
            }

            let target_size = surface_views
                .get(&surface_id)
                .map(|snapshot| snapshot.size)
                .or_else(|| {
                    engine_state
                        .universal_state
                        .surfaces
                        .entries
                        .get(&surface_id)
                        .map(|entry| entry.value.size)
                })
                .unwrap_or(window_state.inner_size);
            let target_format = engine_state
                .universal_state
                .surfaces
                .entries
                .get(&surface_id)
                .and_then(|entry| entry.value.format_policy)
                .unwrap_or(wgpu::TextureFormat::Rgba16Float);

            let (target_view, target_format) = {
                let surface_target = ensure_surface_target(
                    device,
                    &mut engine_state.surface_targets,
                    surface_id,
                    target_size,
                    target_format,
                );
                (surface_target.view.clone(), surface_target.format)
            };

            #[cfg(not(feature = "wasm"))]
            let window_start = std::time::Instant::now();
            #[cfg(feature = "wasm")]
            let window_start = now_ns();

            sync_scene_from_realm_and_universal_resources(
                render_state,
                &engine_state.universal_state,
                *realm_id,
            );
            if synced_windows.insert(*window_id) {
                sync_window_geometry_registry(
                    render_state,
                    &engine_state.universal_state.realm3d.geometries,
                );
            }
            let camera_target_sizes = collect_window_camera_target_sizes(
                &engine_state.universal_state,
                *realm_id,
                *window_id,
                window_state.inner_size,
            );
            if render_state.sync_camera_targets_and_projection(
                device,
                window_state.inner_size,
                Some(&camera_target_sizes),
            ) {
                if let Some(shadow) = render_state.shadow.as_mut() {
                    shadow.mark_dirty();
                }
            }
            render_state.prepare_render(device, frame_spec, true);

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let already_rendered_surface = updated_surfaces.contains(&surface_id);
            let clear_alpha = engine_state
                .universal_state
                .realms
                .entries
                .get(realm_id)
                .map(|entry| vulfram_render::clear_alpha_for_realm_kind(entry.value.kind))
                .unwrap_or(1.0);
            {
                let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Realm Target Clear"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if already_rendered_surface {
                                wgpu::LoadOp::Load
                            } else {
                                wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: clear_alpha,
                                })
                            },
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

            let plan = match resolve_realm_render_graph(&engine_state.universal_state, *realm_id) {
                Some(graph) => graph.plan().clone(),
                None => {
                    log::error!("Realm {} is missing a render graph", realm_id.0);
                    FrameReport::push_unique(&mut frame_report.no_progress_realms, realm_id.0);
                    continue;
                }
            };
            apply_realm_environment_bindings(
                render_state,
                &engine_state.universal_state,
                *realm_id,
                *window_id,
            );
            let universal = &mut engine_state.universal_state;
            let ui_state = &mut universal.ui;
            let targets = &universal.targets;
            let target_layers = &universal.target_layers;
            let surfaces = &universal.surfaces;
            let auto_links = &universal.auto_links;
            #[cfg(not(feature = "wasm"))]
            let window_focused = engine_state
                .window
                .cache
                .caches
                .get(window_id)
                .map(|cache| cache.focused)
                .unwrap_or(true);
            #[cfg(feature = "wasm")]
            let window_focused = true;

            gpu_written |= execute_graph_to_view(
                &plan,
                render_state,
                ui_state,
                *realm_id,
                &mut ui_events,
                &mut ui_platform_actions,
                targets,
                target_layers,
                surfaces,
                auto_links,
                &engine_state.surface_targets,
                device,
                queue,
                &mut encoder,
                &target_view,
                target_format,
                target_size,
                engine_state.runtime.frame.frame_index,
                time as f64,
                *window_id,
                window_focused,
                engine_state.gpu_profiler.as_ref(),
                gpu_base,
                &mut shadow_ns,
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
                &target_view,
                target_format,
                target_size,
                engine_state.runtime.frame.frame_index,
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
            let Some(surface_target) = engine_state.surface_targets.get(&present.value.surface)
            else {
                continue;
            };
            let (window_states, render_states) = (
                &mut engine_state.window.states,
                &mut engine_state.render.states,
            );
            let Some(window_state) = window_states.get_mut(&window_id) else {
                continue;
            };
            let Some(render_state) = render_states.get_mut(&window_id) else {
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
                render_state,
                device,
                queue,
                &mut encoder,
                &surface_view,
                window_state.config.format,
                glam::UVec2::new(window_state.config.width, window_state.config.height),
                &surface_target.view,
                glam::UVec2::new(
                    surface_target.texture.size().width,
                    surface_target.texture.size().height,
                ),
                engine_state.runtime.frame.frame_index,
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

    let soft_cut_diagnostic = build_soft_cut_diagnostic(
        &frame_report,
        previous_cut_edges,
        engine_state.runtime.frame.frame_index,
    );

    engine_state.universal_state.frame_report = frame_report;
    for event in ui_events {
        engine_state
            .runtime
            .event_queue
            .push(crate::core::cmd::EngineEvent::Ui(event));
    }
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
    if let Some(message) = soft_cut_diagnostic {
        push_error_event(engine_state, "realm-graph-soft-cut", message, None, None);
    }
    apply_ui_platform_actions(engine_state, ui_platform_actions);
    engine_state.profiling.render.shadow_ns = shadow_ns;
    engine_state.profiling.render.windows_ns = windows_ns;
    #[cfg(not(feature = "wasm"))]
    {
        engine_state.profiling.render.total_ns = total_start.elapsed().as_nanos() as u64;
    }
    #[cfg(feature = "wasm")]
    {
        engine_state.profiling.render.total_ns = now_ns().saturating_sub(total_start);
    }
}
