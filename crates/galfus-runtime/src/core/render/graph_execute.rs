use crate::core::realm::RealmId;
use crate::core::render::graph::RenderGraphPlan;
use crate::core::render::passes;
use galfus_realm_core::{
    RENDER_PASS_BLOOM, RENDER_PASS_COMPOSE, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL,
    RENDER_PASS_OUTLINE, RENDER_PASS_POST, RENDER_PASS_SHADOW, RENDER_PASS_SKYBOX,
    RENDER_PASS_SSAO, RENDER_PASS_SSAO_BLUR,
};
use std::hash::{DefaultHasher, Hash, Hasher};

use super::RenderState;
use super::frame_helpers::write_gpu_timestamp;

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(target_arch = "wasm32")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

fn encode_custom_param_bytes(
    schema: &std::collections::HashMap<String, String>,
    values: &std::collections::HashMap<String, crate::core::render::graph::RenderGraphValue>,
) -> Result<Vec<u8>, String> {
    let mut fields: Vec<(&str, &str)> = schema
        .iter()
        .map(|(name, ty)| (name.as_str(), ty.as_str()))
        .collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    let mut bytes = Vec::with_capacity(fields.len() * 4 + 16);
    for (name, ty) in fields {
        let value = values
            .get(name)
            .ok_or_else(|| format!("Missing value for shader param '{}'", name))?;
        match (ty, value) {
            ("f32", crate::core::render::graph::RenderGraphValue::Float(v)) => {
                bytes.extend_from_slice(&(*v as f32).to_ne_bytes());
            }
            ("f32", crate::core::render::graph::RenderGraphValue::Int(v)) => {
                bytes.extend_from_slice(&(*v as f32).to_ne_bytes());
            }
            ("i32", crate::core::render::graph::RenderGraphValue::Int(v)) => {
                bytes.extend_from_slice(&(*v as i32).to_ne_bytes());
            }
            ("bool", crate::core::render::graph::RenderGraphValue::Bool(v)) => {
                let raw: u32 = if *v { 1 } else { 0 };
                bytes.extend_from_slice(&raw.to_ne_bytes());
            }
            _ => {
                return Err(format!(
                    "Unsupported shader param '{}' with declared type '{}'",
                    name, ty
                ));
            }
        }
    }
    while bytes.len() % 16 != 0 {
        bytes.push(0);
    }
    Ok(bytes)
}

fn execute_custom_screen_pass(
    node: &crate::core::render::graph::RenderGraphNode,
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    frame_index: u64,
) -> bool {
    let Some(shader) = node.shader.as_ref() else {
        return false;
    };
    if shader.shader_type != crate::core::render::graph::RenderGraphShaderType::Screen {
        return false;
    }

    let generated_wgsl = match crate::core::render::graph::validate_shader_spec(
        shader,
        &node.inputs,
        &node.outputs,
        &node.params,
    ) {
        Ok(source) => source,
        Err(_) => return false,
    };
    let mut hasher = DefaultHasher::new();
    generated_wgsl.hash(&mut hasher);
    let shader_hash = hasher.finish();
    let shader_id = (1u64 << 63) | shader_hash;
    let has_params = !shader.params.is_empty();

    let param_bytes = if has_params {
        match encode_custom_param_bytes(&shader.params, &node.params) {
            Ok(bytes) => Some(bytes),
            Err(_) => return false,
        }
    } else {
        None
    };

    let key = crate::core::render::cache::PipelineKey {
        shader_id,
        color_format: target_format,
        color_target_count: 1,
        depth_format: None,
        sample_count: 1,
        topology: wgpu::PrimitiveTopology::TriangleList,
        polygon_mode: wgpu::PolygonMode::Fill,
        cull_mode: None,
        front_face: wgpu::FrontFace::Ccw,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        blend: None,
    };
    let pipeline = render_state.cache.get_or_create(key, frame_index, || {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Screen Shader"),
            source: wgpu::ShaderSource::Wgsl(generated_wgsl.clone().into()),
        });
        let bind_group_layout = if has_params {
            Some(
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Custom Screen Params BGL"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            )
        } else {
            None
        };
        let bind_group_layouts = match bind_group_layout.as_ref() {
            Some(layout) => vec![layout],
            None => Vec::new(),
        };
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Custom Screen Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            ..Default::default()
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Custom Screen Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    });

    let param_bind_group = if let Some(bytes) = param_bytes.as_ref() {
        let needs_realloc = render_state
            .custom_screen_param_buffer
            .as_ref()
            .map(|buffer| buffer.size() < bytes.len() as u64)
            .unwrap_or(true);
        if needs_realloc {
            render_state.custom_screen_param_buffer =
                Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Custom Screen Params Buffer"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        let Some(buffer) = render_state.custom_screen_param_buffer.as_ref() else {
            return false;
        };
        queue.write_buffer(buffer, 0, bytes);
        let bgl = pipeline.get_bind_group_layout(0);
        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom Screen Params BG"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }))
    } else {
        None
    };

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Custom Screen Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = param_bind_group.as_ref() {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.draw(0..3, 0..1);
    true
}

pub(super) fn execute_graph_to_view(
    plan: &RenderGraphPlan,
    render_state: &mut RenderState,
    _realm_id: RealmId,
    _targets: &crate::core::target::TargetTable,
    _target_layers: &crate::core::target::TargetLayerTable,
    _surfaces: &crate::core::realm::SurfaceTable,
    _target_surface_map: &std::collections::HashMap<
        crate::core::target::TargetId,
        crate::core::realm::SurfaceId,
    >,
    _surface_targets: &std::collections::HashMap<
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
    _time_seconds: f64,
    _window_id: u32,
    _window_focused: bool,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    gpu_base: Option<u32>,
    shadow_cpu_ns_accum: &mut u64,
    log_events: &mut Vec<galfus_log::LogEvent>,
) -> bool {
    let mut gpu_written = false;
    let mut skybox_done = false;
    let pass_order: Vec<&str> = plan
        .order
        .iter()
        .map(|&node_idx| plan.nodes[node_idx].pass_id.as_str())
        .collect();
    galfus_log::galfus_log_debug!(
        log_events,
        "rendergraph.passes",
        "realm={} passes={:?}",
        _realm_id.0,
        pass_order
    );

    for &node_idx in &plan.order {
        let node = &plan.nodes[node_idx];
        galfus_log::galfus_log_debug!(
            log_events,
            "rendergraph.pass",
            "realm={} pass={} node={:?}",
            _realm_id.0,
            node.pass_id,
            node.node_id
        );
        match node.pass_id.as_str() {
            RENDER_PASS_SHADOW => {
                #[cfg(not(target_arch = "wasm32"))]
                let shadow_start = std::time::Instant::now();
                #[cfg(target_arch = "wasm32")]
                let shadow_start = now_ns();
                passes::pass_shadow_update(render_state, device, queue, encoder, frame_index);
                if let Some(shadow) = &mut render_state.shadow {
                    shadow.sync_table();
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    *shadow_cpu_ns_accum = shadow_cpu_ns_accum
                        .saturating_add(shadow_start.elapsed().as_nanos() as u64);
                }
                #[cfg(target_arch = "wasm32")]
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
                    log_events,
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
            _ => {
                if execute_custom_screen_pass(
                    node,
                    render_state,
                    device,
                    queue,
                    encoder,
                    target_view,
                    target_format,
                    frame_index,
                ) {
                    gpu_written = true;
                }
            }
        }
    }

    gpu_written
}
