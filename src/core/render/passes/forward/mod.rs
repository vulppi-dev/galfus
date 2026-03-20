mod branches;
mod collector;
mod draw;

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::SkyboxMode;

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
) {
    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    let camera_ids: Vec<u32> = render_state.camera_order.iter().copied().collect();
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
    let (scene, vertex_sys, bindings, library, light_system, collector, cache, gizmos) = (
        &render_state.scene,
        render_state.vertex.as_mut().unwrap(),
        render_state.bindings.as_mut().unwrap(),
        render_state.library.as_ref().unwrap(),
        render_state.light_system.as_mut().unwrap(),
        &mut render_state.collector,
        &mut render_state.cache,
        &mut render_state.gizmos,
    );

    // 1. Sort cameras by order
    for (camera_index, camera_id) in camera_ids.iter().copied().enumerate() {
        let Some(camera_record) = scene.cameras.get(&camera_id) else {
            continue;
        };
        let sample_count = camera_sample_counts.get(camera_index).copied().unwrap_or(1);
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
        collector::collect_objects(scene, collector, camera_record, vertex_sys);

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
                let camera_offset = bindings.camera_pool.get_offset(camera_id) as u32;
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
                library,
                collector,
                bindings,
                vertex_sys,
                frame_index,
                device,
                cache,
                sample_count,
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
                    topology: wgpu::PrimitiveTopology::LineList,
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
                            topology: wgpu::PrimitiveTopology::LineList,
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
