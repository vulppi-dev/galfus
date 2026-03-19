use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::{SkyboxDirectionalLightSun, SkyboxMode};
use bytemuck::{Pod, Zeroable};

const MAX_SKYBOX_SUNS: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SkyboxUniform {
    inv_view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    intensity: [f32; 4],
    ground_color: [f32; 4],
    horizon_color: [f32; 4],
    sky_color: [f32; 4],
    params: [f32; 4],
    sun_dirs: [[f32; 4]; MAX_SKYBOX_SUNS],
    sun_colors: [[f32; 4]; MAX_SKYBOX_SUNS],
    sun_sizes: [[f32; 4]; MAX_SKYBOX_SUNS],
    sun_meta: [f32; 4],
}

pub fn skybox_uniform_buffer_size() -> u64 {
    std::mem::size_of::<SkyboxUniform>() as u64
}

fn ensure_camera_skybox_msaa_target(
    camera_record: &mut crate::core::resources::CameraRecord,
    device: &wgpu::Device,
    sample_count: u32,
) {
    if sample_count <= 1 {
        camera_record.forward_msaa_target = None;
        return;
    }

    let Some(target) = camera_record.render_target.as_ref() else {
        camera_record.forward_msaa_target = None;
        return;
    };

    let size = target.texture.size();
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
}

fn sanitize_influences(ground_to_horizon: f32, horizon_to_sky: f32) -> (f32, f32) {
    (
        ground_to_horizon.clamp(0.0, 1.0),
        horizon_to_sky.clamp(0.0, 1.0),
    )
}

fn collect_skybox_suns(
    render_state: &RenderState,
    camera_record: &crate::core::resources::CameraRecord,
    directional_lights: &[SkyboxDirectionalLightSun],
) -> (
    [[f32; 4]; MAX_SKYBOX_SUNS],
    [[f32; 4]; MAX_SKYBOX_SUNS],
    [[f32; 4]; MAX_SKYBOX_SUNS],
    u32,
) {
    let mut dirs = [[0.0; 4]; MAX_SKYBOX_SUNS];
    let mut colors = [[0.0; 4]; MAX_SKYBOX_SUNS];
    let mut sizes = [[0.0; 4]; MAX_SKYBOX_SUNS];
    let mut count = 0_u32;

    if directional_lights.is_empty() {
        return (dirs, colors, sizes, count);
    }

    for sun in directional_lights {
        if count as usize >= MAX_SKYBOX_SUNS {
            break;
        }
        let light_id = sun.light_id;
        let Some(light_record) = render_state.scene.lights.get(&light_id) else {
            continue;
        };
        if (light_record.layer_mask & camera_record.layer_mask) == 0 {
            continue;
        }
        let is_directional = light_record.data.kind_flags.x == 0;
        if !is_directional {
            continue;
        }
        let direction = (-light_record.data.direction.truncate()).normalize_or_zero();
        if direction.length_squared() <= f32::EPSILON {
            continue;
        }
        let intensity = light_record.data.intensity_range.x.max(0.0);
        let idx = count as usize;
        dirs[idx] = [direction.x, direction.y, direction.z, 0.0];
        colors[idx] = [
            light_record.data.color.x.max(0.0),
            light_record.data.color.y.max(0.0),
            light_record.data.color.z.max(0.0),
            intensity,
        ];
        let solid = sun.solid_size.max(0.001);
        let gradient = sun.gradient_size.max(solid + 0.001);
        sizes[idx] = [solid, gradient, 0.0, 0.0];
        count += 1;
    }

    (dirs, colors, sizes, count)
}

pub fn pass_skybox(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) -> bool {
    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    let default_skybox = render_state.environment.skybox.clone();
    let camera_skyboxes = render_state.camera_environment_overrides.clone();
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return false,
    };

    let camera_ids: Vec<u32> = render_state.camera_order.iter().copied().collect();
    let mut camera_sample_counts: std::collections::HashMap<u32, u32> =
        std::collections::HashMap::with_capacity(camera_ids.len());
    for camera_id in camera_ids.iter().copied() {
        let sample_count = render_state.msaa_sample_count_for_environment(
            render_state.environment_for_camera(camera_id),
            device,
            TARGET_FORMAT,
        );
        camera_sample_counts.insert(camera_id, sample_count);
    }

    for camera_id in camera_ids.iter().copied() {
        if let Some(camera_record) = render_state.scene.cameras.get_mut(&camera_id) {
            let sample_count = camera_sample_counts.get(&camera_id).copied().unwrap_or(1);
            ensure_camera_skybox_msaa_target(camera_record, device, sample_count);
        }
    }

    let uniform_buffer = match render_state.skybox_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return false,
    };

    let mut drew_any = false;
    for camera_id in render_state.camera_order.iter().copied() {
        let sample_count = camera_sample_counts.get(&camera_id).copied().unwrap_or(1);
        let skybox = camera_skyboxes
            .get(&camera_id)
            .map(|env| env.skybox.clone())
            .unwrap_or_else(|| default_skybox.clone());
        if matches!(skybox.mode, SkyboxMode::None) {
            continue;
        }

        let Some(camera_record) = render_state.scene.cameras.get(&camera_id) else {
            continue;
        };
        let target_view = match &camera_record.render_target {
            Some(target) => &target.view,
            None => continue,
        };

        let (color_view, resolve_target) = if sample_count > 1 {
            match camera_record.forward_msaa_target.as_ref() {
                Some(msaa) => (&msaa.view, Some(target_view)),
                None => (target_view, None),
            }
        } else {
            (target_view, None)
        };

        let target_format = camera_record
            .render_target
            .as_ref()
            .map(|target| target.format)
            .unwrap_or(TARGET_FORMAT);
        let (horizon_ground_threshold, horizon_sky_threshold) = sanitize_influences(
            skybox.horizon_ground_threshold,
            skybox.horizon_sky_threshold,
        );
        let (sun_dirs, sun_colors, sun_sizes, sun_count) =
            collect_skybox_suns(render_state, camera_record, &skybox.directional_lights);

        let pipeline_key = PipelineKey {
            shader_id: ShaderId::Skybox as u64,
            color_format: target_format,
            color_target_count: 1,
            depth_format: None,
            sample_count,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            blend: None,
        };

        let pipeline = render_state
            .cache
            .get_or_create(pipeline_key, frame_index, || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Skybox Pipeline"),
                    layout: Some(&library.skybox_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.skybox_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.skybox_shader,
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
                    multisample: wgpu::MultisampleState {
                        count: sample_count,
                        ..Default::default()
                    },
                    multiview_mask: None,
                    cache: None,
                })
            });

        let inv_view_proj = camera_record.data.view_projection.inverse();
        let camera_pos = camera_record.data.position.truncate();
        let mode_value = match skybox.mode {
            SkyboxMode::None => 0.0,
            SkyboxMode::Procedural => 1.0,
            SkyboxMode::Cubemap => 2.0,
        };

        let uniform = SkyboxUniform {
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            intensity: [skybox.intensity, 0.0, 0.0, 0.0],
            ground_color: [
                skybox.ground_color.x,
                skybox.ground_color.y,
                skybox.ground_color.z,
                1.0,
            ],
            horizon_color: [
                skybox.horizon_color.x,
                skybox.horizon_color.y,
                skybox.horizon_color.z,
                1.0,
            ],
            sky_color: [
                skybox.sky_color.x,
                skybox.sky_color.y,
                skybox.sky_color.z,
                1.0,
            ],
            params: [
                skybox.rotation,
                mode_value,
                horizon_ground_threshold,
                horizon_sky_threshold,
            ],
            sun_dirs,
            sun_colors,
            sun_sizes,
            sun_meta: [sun_count as f32, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let skybox_view = match (skybox.mode, skybox.cubemap_texture_id) {
            (SkyboxMode::Cubemap, Some(id)) => render_state
                .scene
                .textures
                .get(&id)
                .map(|record| &record.view)
                .unwrap_or(&library.fallback_view),
            _ => &library.fallback_view,
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skybox Bind Group"),
            layout: &library.layout_skybox,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(skybox_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Skybox Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        drew_any = true;
    }

    drew_any
}
