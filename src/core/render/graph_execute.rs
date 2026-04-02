use crate::core::realm::RealmId;
use crate::core::render::graph::RenderGraphPlan;
use crate::core::render::passes;
use crate::core::render::passes::UiPlatformAction;
use crate::core::ui::events::UiEvent;
use vulfram_realm_core::{
    RENDER_PASS_BLOOM, RENDER_PASS_COMPOSE, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL,
    RENDER_PASS_OUTLINE, RENDER_PASS_POST, RENDER_PASS_SHADOW, RENDER_PASS_SKYBOX,
    RENDER_PASS_SSAO, RENDER_PASS_SSAO_BLUR, RENDER_PASS_UI,
};

use super::RenderState;
use super::frame_helpers::write_gpu_timestamp;

#[cfg(feature = "wasm")]
use js_sys::Date;

#[cfg(feature = "wasm")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

pub(super) fn execute_graph_to_view(
    plan: &RenderGraphPlan,
    render_state: &mut RenderState,
    ui_state: &mut crate::core::ui::UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    ui_platform_actions: &mut Vec<UiPlatformAction>,
    targets: &crate::core::target::TargetTable,
    target_layers: &crate::core::target::TargetLayerTable,
    surfaces: &crate::core::realm::SurfaceTable,
    auto_links: &std::collections::HashMap<
        (u32, crate::core::target::TargetId),
        crate::core::realm::AutoLink,
    >,
    surface_targets: &std::collections::HashMap<
        crate::core::realm::SurfaceId,
        crate::core::resources::RenderTarget,
    >,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    time_seconds: f64,
    window_id: u32,
    window_focused: bool,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    gpu_base: Option<u32>,
    shadow_cpu_ns_accum: &mut u64,
) -> bool {
    let mut gpu_written = false;
    let mut skybox_done = false;

    for &node_idx in &plan.order {
        let node = &plan.nodes[node_idx];
        match node.pass_id.as_str() {
            RENDER_PASS_SHADOW => {
                #[cfg(not(feature = "wasm"))]
                let shadow_start = std::time::Instant::now();
                #[cfg(feature = "wasm")]
                let shadow_start = now_ns();
                passes::pass_shadow_update(render_state, device, queue, encoder, frame_index);
                if let Some(shadow) = &mut render_state.shadow {
                    shadow.sync_table();
                }
                #[cfg(not(feature = "wasm"))]
                {
                    *shadow_cpu_ns_accum = shadow_cpu_ns_accum
                        .saturating_add(shadow_start.elapsed().as_nanos() as u64);
                }
                #[cfg(feature = "wasm")]
                {
                    *shadow_cpu_ns_accum =
                        shadow_cpu_ns_accum.saturating_add(now_ns().saturating_sub(shadow_start));
                }
            }
            RENDER_PASS_SKYBOX => {
                skybox_done =
                    passes::pass_skybox(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_LIGHT_CULL => {
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base, &mut gpu_written);
                }
                passes::pass_light_cull(render_state, device, encoder, frame_index);
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 1, &mut gpu_written);
                }
            }
            RENDER_PASS_FORWARD => {
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
            RENDER_PASS_OUTLINE => {
                passes::pass_outline(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_SSAO => {
                passes::pass_ssao(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_SSAO_BLUR => {
                passes::pass_ssao_blur(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_BLOOM => {
                passes::pass_bloom(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_POST => {
                passes::pass_post(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_COMPOSE => {
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
            RENDER_PASS_UI => {
                let mut actions = passes::pass_ui(
                    render_state,
                    ui_state,
                    realm_id,
                    window_id,
                    window_focused,
                    ui_events,
                    targets,
                    target_layers,
                    surfaces,
                    auto_links,
                    surface_targets,
                    device,
                    queue,
                    encoder,
                    target_view,
                    target_format,
                    target_size,
                    frame_index,
                    time_seconds,
                );
                ui_platform_actions.append(&mut actions);
                gpu_written = true;
            }
            _ => {}
        }
    }

    gpu_written
}
