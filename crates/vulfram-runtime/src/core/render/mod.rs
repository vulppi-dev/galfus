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
use crate::core::profiling::gpu::apply_gpu_timing_report;
use crate::core::realm::{FrameReport, apply_target_graph_stats};
use crate::core::render::passes::UiPlatformAction;
use crate::core::resources::RenderTarget;
use crate::core::state::EngineState;
use crate::core::ui::events::UiEvent;
use frame_helpers::{
    apply_realm_environment_bindings, apply_target_size_requests, build_target_surface_map,
    collect_window_camera_target_sizes, refresh_window_target_textures, should_render_realm,
};
use graph_execute::execute_graph_to_view;
use realm_graph::{
    collect_present_sizes, collect_surface_views, ensure_surface_target, map_realms_to_windows,
    resolve_realm_surface,
};
pub use runtime::RenderManager;
use scene_sync::{sync_scene_from_realm_and_universal_resources, sync_window_geometry_registry};
pub use state::{
    Realm3dState, RealmEntities, RenderCatalogState, RenderResourceState, RenderState,
    SceneRuntimeState, UniversalGeometryRecord,
};
use std::collections::HashSet;
use ui_platform_actions::apply_ui_platform_actions;
pub fn bloom_chain_size(base: u32, level: usize) -> u32 {
    passes::bloom_chain_size(base, level)
}

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(target_arch = "wasm32")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

fn resolve_realm_render_graph<'a>(
    universal: &'a crate::core::realm::UniversalState,
    realm_id: crate::core::realm::RealmId,
) -> Option<&'a crate::core::render::graph::RenderGraphState> {
    let realm = universal.composition.realms.entries.get(&realm_id)?;
    let realm_kind = realm.value.kind;
    let render_graph_id =
        vulfram_render::resolve_render_graph_id(realm.value.render_graph_id, realm_kind);
    let registry = match realm_kind {
        crate::core::realm::RealmKind::ThreeD => &universal.render_catalog.render_graphs_3d,
        crate::core::realm::RealmKind::TwoD => &universal.render_catalog.render_graphs_2d,
    };
    if let Some(graph) = registry.get(&render_graph_id) {
        return Some(&graph.state);
    }
    None
}

pub fn ensure_runtime_render_defaults(universal: &mut crate::core::realm::UniversalState) {
    let fallback_3d = crate::core::render::graph::fallback_graph();
    let hash_3d = crate::core::render::graph::render_graph_desc_hash(&fallback_3d);
    let state_3d = universal
        .render_catalog
        .render_graph_plan_cache_3d
        .entry(hash_3d)
        .or_default()
        .clone();
    universal
        .render_catalog
        .render_graphs_3d
        .entry(crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID)
        .or_insert(crate::core::render::graph::RenderGraphRecord {
            state: state_3d,
            desc_hash: hash_3d,
        });

    let fallback_2d = crate::core::render::graph::ui_fallback_graph();
    let hash_2d = crate::core::render::graph::render_graph_desc_hash(&fallback_2d);
    let state_2d = universal
        .render_catalog
        .render_graph_plan_cache_2d
        .entry(hash_2d)
        .or_insert_with(crate::core::render::graph::RenderGraphState::new_ui)
        .clone();
    universal
        .render_catalog
        .render_graphs_2d
        .entry(crate::core::render::graph::DEFAULT_2D_RENDER_GRAPH_ID)
        .or_insert(crate::core::render::graph::RenderGraphRecord {
            state: state_2d,
            desc_hash: hash_2d,
        });
    universal
        .scene
        .realm3d
        .materials
        .entry(crate::core::resources::MATERIAL_FALLBACK_ID)
        .or_insert_with(|| {
            crate::core::resources::ShaderMaterialRecord::new_standard(Some(
                "Fallback Material".into(),
            ))
        });
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

    let target_size_requests = std::mem::take(
        &mut engine_state
            .universal_state
            .interaction
            .ui
            .target_size_requests,
    );
    apply_target_size_requests(engine_state, &target_size_requests);

    let (target_plan, target_diff) = {
        let target_dependencies = crate::core::target::collect_target_dependencies(
            &engine_state.universal_state.targets.target_layers.entries,
            &engine_state.universal_state.scene,
        );
        let cache = &mut engine_state.universal_state.targets.target_graph_cache;
        let diff = cache
            .update(
                &engine_state.universal_state.targets.targets.entries,
                &target_dependencies,
                &engine_state.universal_state.targets.target_layers.entries,
                &engine_state.universal_state.composition.realms,
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

    let time = engine_state.runtime.time_ms() as f32 / 1000.0;
    let delta_time = engine_state.runtime.delta_time_ms() as f32 / 1000.0;
    let frame_index = engine_state.runtime.frame_index() as u32;
    let frame_spec = crate::core::resources::FrameComponent::new(time, delta_time, frame_index);
    let mut gpu_written = false;

    #[cfg(not(target_arch = "wasm32"))]
    let total_start = std::time::Instant::now();
    #[cfg(target_arch = "wasm32")]
    let total_start = now_ns();

    // 1. Render realms in target-scheduler order
    let mut windows_ns: u64 = 0;
    let mut shadow_ns: u64 = 0;
    let mut frame_report = FrameReport::default();
    apply_target_graph_stats(&mut frame_report, &target_plan, target_diff.as_ref());
    let window_sizes: std::collections::HashMap<u32, glam::UVec2> = engine_state
        .window
        .states
        .iter()
        .map(|(window_id, state)| (*window_id, state.inner_size))
        .collect();
    let target_invocations = crate::core::target::collect_render_invocations(
        &target_plan.order,
        &engine_state.universal_state.targets.targets.entries,
        &engine_state.universal_state.targets.target_layers.entries,
        &window_sizes,
        engine_state.runtime.frame_index(),
    );
    frame_report.target_invocations = target_invocations
        .iter()
        .map(|invocation| crate::core::realm::TargetInvocationReport {
            realm_id: invocation.realm_id,
            target_id: invocation.target_id.0,
            rect_px: [
                invocation.resolved_rect_px.x,
                invocation.resolved_rect_px.y,
                invocation.resolved_rect_px.z,
                invocation.resolved_rect_px.w,
            ],
            render_size_px: [invocation.render_size_px.x, invocation.render_size_px.y],
            frame_id: invocation.frame_id,
        })
        .collect();
    frame_report.target_autolink_failures = engine_state
        .universal_state
        .targets
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
    let surface_views = collect_surface_views(
        device,
        &engine_state.universal_state,
        &mut engine_state.surface_targets,
        present_sizes,
    );
    let target_surface_map = build_target_surface_map(
        &engine_state.universal_state.targets.targets,
        &engine_state.universal_state.targets.auto_links,
    );
    let blocked_target_ids: HashSet<_> = target_plan
        .cut_edges
        .iter()
        .map(|edge| edge.parent)
        .collect();
    refresh_window_target_textures(
        &mut engine_state.render.states,
        &engine_state
            .universal_state
            .scene
            .render_resources
            .target_texture_binds,
        &blocked_target_ids,
        &target_surface_map,
        &engine_state.surface_targets,
    );
    let mut updated_surfaces: HashSet<crate::core::realm::SurfaceId> = HashSet::new();
    let mut invocation_targets: std::collections::HashMap<(u64, u32), RenderTarget> =
        std::collections::HashMap::new();
    let mut ui_events: Vec<UiEvent> = Vec::new();
    let mut ui_platform_actions: Vec<UiPlatformAction> = Vec::new();
    let mut synced_windows: HashSet<u32> = HashSet::new();
    const MAX_REALM_ITERATIONS: u32 = 1;
    let mut iteration: u32 = 0;
    loop {
        frame_report.no_progress_realms.clear();
        let mut window_counter: u32 = 0;

        for invocation in &target_invocations {
            let realm_id = crate::core::realm::RealmId(invocation.realm_id);
            let target_id = invocation.target_id;
            let target_window_id = engine_state
                .universal_state
                .targets
                .targets
                .entries
                .get(&target_id)
                .and_then(|target| target.window_id);
            let Some(window_id) =
                target_window_id.or_else(|| realm_windows.get(&realm_id).copied())
            else {
                continue;
            };
            let Some(window_state) = engine_state.window.states.get(&window_id) else {
                continue;
            };
            let Some(render_state) = engine_state.render.get_mut(&window_id) else {
                continue;
            };
            let Some(layer_state) = engine_state
                .universal_state
                .targets
                .target_layers
                .entries
                .get(&invocation.layer_key)
            else {
                continue;
            };
            let layer_blend_mode = layer_state.layout.blend_mode;
            let layer_clip = layer_state.layout.clip;
            let layer_opacity = layer_state.layout.opacity;
            let Some(surface_id) = target_surface_map
                .get(&target_id)
                .copied()
                .or_else(|| resolve_realm_surface(&engine_state.universal_state, realm_id))
            else {
                continue;
            };
            let should_render = engine_state
                .universal_state
                .composition
                .realms
                .entries
                .get_mut(&realm_id)
                .map(|realm_entry| {
                    should_render_realm(realm_entry, engine_state.runtime.frame_index())
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
                        .composition
                        .surfaces
                        .entries
                        .get(&surface_id)
                        .map(|entry| entry.value.size)
                })
                .unwrap_or(window_state.inner_size);
            let target_format = engine_state
                .universal_state
                .composition
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
            let invocation_size = invocation.render_size_px;
            let invocation_target = vulfram_render::ensure_surface_target(
                device,
                &mut invocation_targets,
                (target_id.0, invocation.realm_id),
                invocation_size,
                target_format,
            );
            let invocation_view = invocation_target.view.clone();

            #[cfg(not(target_arch = "wasm32"))]
            let window_start = std::time::Instant::now();
            #[cfg(target_arch = "wasm32")]
            let window_start = now_ns();

            sync_scene_from_realm_and_universal_resources(
                render_state,
                &engine_state.universal_state,
                realm_id,
            );
            if synced_windows.insert(window_id) {
                sync_window_geometry_registry(
                    render_state,
                    &engine_state.universal_state.scene.realm3d.geometries,
                );
            }
            let camera_target_sizes = collect_window_camera_target_sizes(
                &engine_state.universal_state,
                realm_id,
                window_id,
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
                .composition
                .realms
                .entries
                .get(&realm_id)
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

            let plan = match resolve_realm_render_graph(&engine_state.universal_state, realm_id) {
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
                realm_id,
                window_id,
            );
            let universal = &mut engine_state.universal_state;
            let ui_state = &mut universal.interaction.ui;
            let targets = &universal.targets.targets;
            let target_layers = &universal.targets.target_layers;
            let surfaces = &universal.composition.surfaces;
            let auto_links = &universal.targets.auto_links;
            #[cfg(not(target_arch = "wasm32"))]
            let window_focused = engine_state
                .window
                .cache
                .caches
                .get(&window_id)
                .map(|cache| cache.focused)
                .unwrap_or(true);
            #[cfg(target_arch = "wasm32")]
            let window_focused = true;

            gpu_written |= execute_graph_to_view(
                &plan,
                render_state,
                ui_state,
                realm_id,
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
                &invocation_view,
                target_format,
                invocation_size,
                engine_state.runtime.frame_index(),
                time as f64,
                window_id,
                window_focused,
                engine_state.gpu_profiler.as_ref(),
                gpu_base,
                &mut shadow_ns,
            );
            let overlay_blend = match layer_blend_mode {
                1 => Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                2 => None,
                _ => Some(wgpu::BlendState::ALPHA_BLENDING),
            };
            let overlay = [passes::ComposeOverlay {
                source_view: &invocation_view,
                source_size: invocation_size,
                rect: glam::Vec4::new(
                    invocation.resolved_rect_px.x as f32,
                    invocation.resolved_rect_px.y as f32,
                    invocation.resolved_rect_px.z as f32,
                    invocation.resolved_rect_px.w as f32,
                ),
                clip: layer_clip,
                blend: overlay_blend,
                opacity: layer_opacity,
            }];
            passes::pass_compose_overlays(
                render_state,
                device,
                queue,
                &mut encoder,
                &target_view,
                target_format,
                target_size,
                &overlay,
                engine_state.runtime.frame_index(),
            );

            updated_surfaces.insert(surface_id);

            queue.submit(Some(encoder.finish()));
            #[cfg(not(target_arch = "wasm32"))]
            {
                windows_ns = windows_ns.saturating_add(window_start.elapsed().as_nanos() as u64);
            }
            #[cfg(target_arch = "wasm32")]
            {
                windows_ns = windows_ns.saturating_add(now_ns().saturating_sub(window_start));
            }
        }

        for present in engine_state
            .universal_state
            .composition
            .presents
            .entries
            .values()
        {
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
                engine_state.runtime.frame_index(),
            );

            queue.submit(Some(encoder.finish()));
            surface_texture.present();
            #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(target_arch = "wasm32")]
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

    engine_state.universal_state.composition.frame_report = frame_report;
    for event in ui_events {
        engine_state
            .runtime
            .push_event(crate::core::cmd::EngineEvent::Ui(event));
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
                if let Some(report) = gpu_profiler.readback_report(device) {
                    apply_gpu_timing_report(&mut engine_state.profiling, report);
                }
            }
        }
    }
    apply_ui_platform_actions(engine_state, ui_platform_actions);
    engine_state.profiling.render.shadow_ns = shadow_ns;
    engine_state.profiling.render.windows_ns = windows_ns;
    #[cfg(not(target_arch = "wasm32"))]
    {
        engine_state.profiling.render.total_ns = total_start.elapsed().as_nanos() as u64;
    }
    #[cfg(target_arch = "wasm32")]
    {
        engine_state.profiling.render.total_ns = now_ns().saturating_sub(total_start);
    }
}
