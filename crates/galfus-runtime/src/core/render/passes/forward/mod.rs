mod branches;
mod collector;
mod draw;

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::SkyboxMode;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ForwardFrameSemanticMeta {
    resolution: glam::Vec2,
    inv_resolution: glam::Vec2,
    frame_index: u32,
    flags: u32,
}

const FWD_SEM_SCENE_COLOR: u32 = 1 << 0;
const FWD_SEM_SCENE_DEPTH: u32 = 1 << 1;
const FWD_SEM_HISTORY0: u32 = 1 << 2;
const FWD_SEM_HISTORY1: u32 = 1 << 3;

fn ensure_camera_forward_targets(
    camera_record: &mut crate::core::resources::CameraRecord,
    device: &wgpu::Device,
    sample_count: u32,
) {
    let Some(target) = camera_record.render_target.as_ref() else {
        return;
    };

    let size = target.texture.size();
    let needs_depth = match camera_record.forward_depth_target.as_ref() {
        Some(existing) => {
            let existing_size = existing.texture.size();
            existing_size.width != size.width
                || existing_size.height != size.height
                || existing.sample_count != sample_count
        }
        None => true,
    };
    if needs_depth {
        camera_record.forward_depth_target =
            Some(crate::core::resources::RenderTarget::new_with_samples(
                device,
                size,
                wgpu::TextureFormat::Depth32Float,
                sample_count,
            ));
    }

    if sample_count > 1 {
        let needs_msaa = match camera_record.forward_msaa_target.as_ref() {
            Some(existing) => {
                let existing_size = existing.texture.size();
                existing_size.width != size.width
                    || existing_size.height != size.height
                    || existing.sample_count != sample_count
            }
            None => true,
        };
        if needs_msaa {
            camera_record.forward_msaa_target =
                Some(crate::core::resources::RenderTarget::new_with_samples(
                    device,
                    size,
                    wgpu::TextureFormat::Rgba16Float,
                    sample_count,
                ));
        }

        let needs_emissive_msaa = match camera_record.forward_emissive_msaa_target.as_ref() {
            Some(existing) => {
                let existing_size = existing.texture.size();
                existing_size.width != size.width
                    || existing_size.height != size.height
                    || existing.sample_count != sample_count
            }
            None => true,
        };
        if needs_emissive_msaa {
            camera_record.forward_emissive_msaa_target =
                Some(crate::core::resources::RenderTarget::new_with_samples(
                    device,
                    size,
                    wgpu::TextureFormat::Rgba16Float,
                    sample_count,
                ));
        }
    } else {
        camera_record.forward_msaa_target = None;
        camera_record.forward_emissive_msaa_target = None;
    }
}

pub fn pass_forward(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
    clear_color: bool,
    log_events: &mut Vec<galfus_log::LogEvent>,
) {
    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    let camera_ids: Vec<u32> = render_state.camera_order.iter().copied().collect();
    galfus_log::galfus_log_debug!(
        log_events,
        "forward.pass",
        "frame={} cameras={} lights={} models={} materials={} has_shared_group={}",
        frame_index,
        camera_ids.len(),
        render_state.scene.lights.len(),
        render_state.scene.models.len(),
        render_state.scene.materials.len(),
        render_state
            .bindings
            .as_ref()
            .and_then(|b| b.shared_group.as_ref())
            .is_some()
    );
    let camera_slots: Vec<Option<u32>> = camera_ids
        .iter()
        .map(|camera_id| render_state.camera_uniform_slot(*camera_id))
        .collect();
    let mut camera_sample_counts: Vec<u32> = Vec::with_capacity(camera_ids.len());
    let mut camera_clear_colors: Vec<glam::Vec4> = Vec::with_capacity(camera_ids.len());
    let mut camera_skybox_modes: Vec<SkyboxMode> = Vec::with_capacity(camera_ids.len());
    for camera_id in camera_ids.iter().copied() {
        let environment = render_state.environment_for_camera(camera_id);
        let sample_count =
            render_state.msaa_sample_count_for_environment(environment, device, TARGET_FORMAT);
        camera_sample_counts.push(sample_count);
        camera_clear_colors.push(environment.clear_color);
        camera_skybox_modes.push(environment.skybox.mode);
        if let Some(camera_record) = render_state.scene.cameras.get_mut(&camera_id) {
            ensure_camera_forward_targets(camera_record, device, sample_count);
        }
    }

    // Split borrows
    let (
        scene,
        vertex_sys,
        bindings,
        library,
        light_system,
        collector,
        cache,
        gizmos,
        material_shader_modules,
    ) = (
        &render_state.scene,
        render_state.vertex.as_mut().unwrap(),
        render_state.bindings.as_mut().unwrap(),
        render_state.library.as_ref().unwrap(),
        render_state.light_system.as_mut().unwrap(),
        &mut render_state.collector,
        &mut render_state.cache,
        &mut render_state.gizmos,
        &mut render_state.material_shader_modules,
    );
    let fallback_depth_view =
        library
            ._fallback_shadow_texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("Forward Fallback Depth View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                usage: None,
            });

    // 1. Sort cameras by order
    for (camera_index, camera_id) in camera_ids.iter().copied().enumerate() {
        let Some(camera_record) = scene.cameras.get(&camera_id) else {
            continue;
        };
        let sample_count = camera_sample_counts.get(camera_index).copied().unwrap_or(1);
        galfus_log::galfus_log_debug!(
            log_events,
            "forward.camera",
            "camera={} slot={:?} sample_count={} layer_mask={} order={} has_rt={} has_depth={} rt={}x{} pos=({:.3},{:.3},{:.3}) dir=({:.3},{:.3},{:.3})",
            camera_id,
            camera_slots.get(camera_index).copied().flatten(),
            sample_count,
            camera_record.layer_mask,
            camera_record.order,
            camera_record.render_target.is_some(),
            camera_record.forward_depth_target.is_some(),
            camera_record
                .render_target
                .as_ref()
                .map(|t| t.texture.size().width)
                .unwrap_or(0),
            camera_record
                .render_target
                .as_ref()
                .map(|t| t.texture.size().height)
                .unwrap_or(0),
            camera_record.data.position.x,
            camera_record.data.position.y,
            camera_record.data.position.z,
            camera_record.data.direction.x,
            camera_record.data.direction.y,
            camera_record.data.direction.z
        );
        light_system.write_draw_params(camera_index as u32, light_system.max_lights_per_camera);

        let clear_rgb = camera_clear_colors
            .get(camera_index)
            .copied()
            .unwrap_or(glam::Vec4::ZERO);
        let has_skybox_for_camera = !matches!(
            camera_skybox_modes
                .get(camera_index)
                .copied()
                .unwrap_or(SkyboxMode::None),
            SkyboxMode::None
        );
        let clear_wgpu = wgpu::Color {
            r: clear_rgb.x as f64,
            g: clear_rgb.y as f64,
            b: clear_rgb.z as f64,
            a: clear_rgb.w as f64,
        };
        let Some(camera_slot) = camera_slots.get(camera_index).copied().flatten() else {
            continue;
        };

        // 2. Get render target view
        let target_view = match &camera_record.render_target {
            Some(target) => &target.view,
            None => continue,
        };
        let emissive_target = match &camera_record.emissive_target {
            Some(target) => target,
            None => continue,
        };
        let depth_target = match camera_record.forward_depth_target.as_ref() {
            Some(target) => target,
            None => continue,
        };
        let scene_color_view = camera_record
            .post_target
            .as_ref()
            .or(camera_record.render_target.as_ref())
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let scene_depth_view = if depth_target.sample_count == 1 {
            &depth_target.view
        } else {
            &fallback_depth_view
        };
        let history0_view = camera_record
            .history0_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let history1_view = camera_record
            .history1_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let mut semantics_flags = FWD_SEM_SCENE_COLOR;
        if depth_target.sample_count == 1 {
            semantics_flags |= FWD_SEM_SCENE_DEPTH;
        }
        if camera_record.history0_target.is_some() && camera_record.history_valid {
            semantics_flags |= FWD_SEM_HISTORY0;
        }
        if camera_record.history1_target.is_some() && camera_record.history_valid {
            semantics_flags |= FWD_SEM_HISTORY1;
        }
        let target_size = camera_record
            .render_target
            .as_ref()
            .map(|target| target.texture.size())
            .unwrap_or(wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            });
        let w = target_size.width.max(1) as f32;
        let h = target_size.height.max(1) as f32;
        let meta = ForwardFrameSemanticMeta {
            resolution: glam::Vec2::new(w, h),
            inv_resolution: glam::Vec2::new(1.0 / w, 1.0 / h),
            frame_index: frame_index as u32,
            flags: semantics_flags,
        };
        let meta_bytes = bytemuck::bytes_of(&meta);
        let needs_realloc = render_state
            .forward_semantics_buffer
            .as_ref()
            .map(|buffer| buffer.size() < meta_bytes.len() as u64)
            .unwrap_or(true);
        if needs_realloc {
            render_state.forward_semantics_buffer =
                Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Forward Semantics Buffer"),
                    size: meta_bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        let Some(forward_semantics_buffer) = render_state.forward_semantics_buffer.as_ref() else {
            continue;
        };
        queue.write_buffer(forward_semantics_buffer, 0, meta_bytes);
        let forward_semantics_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Forward Frame Semantics BG"),
            layout: &library.layout_frame_semantics,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(history0_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(history1_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: forward_semantics_buffer.as_entire_binding(),
                },
            ],
        });
        let msaa_target = if sample_count > 1 {
            camera_record.forward_msaa_target.as_ref()
        } else {
            None
        };
        let emissive_msaa_target = if sample_count > 1 {
            camera_record.forward_emissive_msaa_target.as_ref()
        } else {
            None
        };
        let (color_view, resolve_target) = if let Some(msaa) = msaa_target {
            (&msaa.view, Some(target_view))
        } else {
            (target_view, None)
        };
        let (emissive_view, emissive_resolve) = if let Some(msaa) = emissive_msaa_target {
            (&msaa.view, Some(&emissive_target.view))
        } else {
            (&emissive_target.view, None)
        };

        // Reset collector for this camera
        collector.clear();

        // 3. Collection & Sorting
        collector::collect_objects(scene, collector, camera_record, vertex_sys, log_events);

        // 4. Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Forward Pass - Camera {}", camera_id)),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target,
                        ops: wgpu::Operations {
                            load: if clear_color {
                                wgpu::LoadOp::Clear(clear_wgpu)
                            } else if has_skybox_for_camera {
                                wgpu::LoadOp::Load
                            } else {
                                wgpu::LoadOp::Clear(clear_wgpu)
                            },
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: emissive_view,
                        resolve_target: emissive_resolve,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_target.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0), // Reverse Z
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            vertex_sys.begin_pass();

            // 5. Bind Shared (Group 0: Frame + Camera + ModelPool)
            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(camera_slot) as u32;
                let light_offset = light_system.draw_params_offset(camera_index as u32) as u32;
                render_pass.set_bind_group(0, shared_group, &[camera_offset, light_offset]);
            }

            // Write instances
            if !collector.instance_data.is_empty() {
                bindings
                    .instance_pool
                    .write_slice(0, &collector.instance_data);
            }

            // 6. Draw Batches
            draw::draw_batches(
                &mut render_pass,
                scene,
                &render_state.material_uniform_slots,
                library,
                collector,
                bindings,
                vertex_sys,
                frame_index,
                device,
                cache,
                sample_count,
                material_shader_modules,
                &forward_semantics_bind_group,
                log_events,
            );

            // 7. Draw Gizmos
            if !gizmos.is_empty() {
                let viewport_size = camera_record
                    .render_target
                    .as_ref()
                    .map(|target| {
                        let size = target.texture.size();
                        glam::UVec2::new(size.width, size.height)
                    })
                    .unwrap_or(glam::UVec2::new(1, 1));
                gizmos.prepare_for_camera(device, queue, camera_record, viewport_size);
                let key = PipelineKey {
                    shader_id: ShaderId::Gizmo as u64,
                    color_format: TARGET_FORMAT,
                    color_target_count: 2,
                    depth_format: Some(wgpu::TextureFormat::Depth32Float),
                    sample_count,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    cull_mode: None,
                    front_face: wgpu::FrontFace::Ccw,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Greater,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                };
                let pipeline = cache.get_or_create(key, frame_index, || {
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Gizmo Pipeline"),
                        layout: Some(&library.gizmo_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &library.gizmo_shader,
                            entry_point: Some("vs_main"),
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: std::mem::size_of::<
                                    crate::core::render::gizmos::GizmoVertex,
                                >() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x4,
                                        offset: 16,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &library.gizmo_shader,
                            entry_point: Some("fs_main"),
                            targets: &[
                                Some(wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Rgba16Float,
                                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask: wgpu::ColorWrites::ALL,
                                }),
                                Some(wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Rgba16Float,
                                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask: wgpu::ColorWrites::empty(),
                                }),
                            ],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: depth_target.format,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Greater,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState {
                            count: sample_count,
                            ..Default::default()
                        },
                        multiview_mask: None,
                        cache: None,
                    })
                });
                render_pass.set_pipeline(pipeline);
                gizmos.draw(&mut render_pass);
            }
        }
    }
}
