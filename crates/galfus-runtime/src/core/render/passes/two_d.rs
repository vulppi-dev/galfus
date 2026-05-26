use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::render::state::{
    TwoDBatchKey, TwoDBatchRange, TwoDItemKind, TwoDPreparedCamera, TwoDPreparedItem,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDCameraRaw {
    view_projection: glam::Mat4,
    tint: glam::Vec4,
    model_position: glam::Vec4,
    light_offset_count: glam::UVec4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDFrameSemanticMeta {
    resolution: glam::Vec2,
    inv_resolution: glam::Vec2,
    frame_index: u32,
    flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDLightRaw {
    position: glam::Vec4,
    color: glam::Vec4,
    intensity_range: glam::Vec2,
    kind_flags: glam::UVec2,
}

const TWO_D_MAX_LIGHTS_PER_CAMERA: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoDDrawBatch {
    key: TwoDBatchKey,
    start: u32,
    count: u32,
}

fn material_allows_2d(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    matches!(
        record.realm_kind,
        crate::core::resources::MaterialRealmKind::TwoD
    )
}

fn material_uses_custom_2d_shader(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    material_allows_2d(record)
        && record.compile_error.is_none()
        && record.compiled_shader_source.is_some()
}

fn resolve_2d_draw_batches<FMat, FGeom>(
    ranges: &[TwoDBatchRange],
    mut material_exists: FMat,
    mut geometry_exists: FGeom,
) -> Vec<TwoDDrawBatch>
where
    FMat: FnMut(u32) -> bool,
    FGeom: FnMut(u32) -> bool,
{
    let mut batches = Vec::with_capacity(ranges.len());
    for range in ranges {
        if range.count == 0 {
            continue;
        }
        if !material_exists(range.key.material_id) {
            continue;
        }
        if !geometry_exists(range.key.geometry_id) {
            continue;
        }
        batches.push(TwoDDrawBatch {
            key: range.key,
            start: range.start,
            count: range.count,
        });
    }
    batches
}

fn material_tint_for_batch(
    scene: &crate::core::render::state::RenderScene,
    material_id: u32,
) -> glam::Vec4 {
    let Some(material) = scene.materials.get(&material_id) else {
        return glam::Vec4::ONE;
    };
    if let Some(input_tint) = material.inputs.first().copied()
        && input_tint.w > 0.0
    {
        return input_tint;
    }
    glam::Vec4::ONE
}

fn align_up(value: u64, alignment: u64) -> u64 {
    if alignment <= 1 {
        return value;
    }
    value.div_ceil(alignment) * alignment
}

fn collect_visible_2d_lights(
    render_state: &RenderState,
    camera: &crate::core::render::state::TwoDPreparedCamera,
) -> Vec<TwoDLightRaw> {
    let mut visible_lights = Vec::with_capacity(TWO_D_MAX_LIGHTS_PER_CAMERA);
    let camera_position = camera.transform.w_axis.truncate();
    for light in render_state.scene.lights.values() {
        if !light.active || (light.layer_mask & camera.layer_mask) == 0 {
            continue;
        }
        let light_kind = light.data.kind_flags.x;
        if light_kind == crate::core::resources::LightKind::Point.to_u32()
            || light_kind == crate::core::resources::LightKind::Spot.to_u32()
        {
            let range = light.data.intensity_range.y.max(0.0001);
            let delta = light.data.position.truncate() - camera_position;
            if delta.length_squared() > (range * range * 4.0) {
                continue;
            }
        }
        visible_lights.push(TwoDLightRaw {
            position: light.data.position,
            color: light.data.color,
            intensity_range: light.data.intensity_range,
            kind_flags: light.data.kind_flags,
        });
        if visible_lights.len() >= TWO_D_MAX_LIGHTS_PER_CAMERA {
            break;
        }
    }
    visible_lights
}

fn ensure_two_d_pass_resources(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    required_slots: usize,
) {
    let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
    let stride = align_up(std::mem::size_of::<TwoDCameraRaw>() as u64, min_alignment);
    let initial_slots = required_slots.max(1);
    let initial_light_slots = TWO_D_MAX_LIGHTS_PER_CAMERA.max(1);
    let resources = render_state.two_d_pass_resources.get_or_insert_with(|| {
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("2D Global BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                                TwoDCameraRaw,
                            >(
                            )
                                as u64),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let library = render_state.library.as_ref().expect("library must exist");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Material Pipeline Layout"),
            bind_group_layouts: &[
                &global_bind_group_layout,
                &library.layout_object_3d_material,
                &library.layout_frame_semantics,
            ],
            ..Default::default()
        });
        let camera_dynamic_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Camera Dynamic Buffer"),
            size: stride * initial_slots as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Light Storage Buffer"),
            size: (std::mem::size_of::<TwoDLightRaw>() * initial_light_slots) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("2D Global BG"),
                layout: &global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &camera_dynamic_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<TwoDCameraRaw>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: light_storage_buffer.as_entire_binding(),
                    },
                ],
            });
        let fallback_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("2D Fallback Depth Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &fallback_depth,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::bytes_of(&1.0f32),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let fallback_depth_view =
            fallback_depth.create_view(&wgpu::TextureViewDescriptor::default());
        crate::core::render::state::TwoDPassResources {
            global_bind_group_layout,
            pipeline_layout,
            camera_dynamic_buffer,
            light_storage_buffer,
            global_bind_group,
            camera_dynamic_stride: stride,
            camera_dynamic_capacity_slots: initial_slots,
            light_capacity_slots: initial_light_slots,
            fallback_depth_view,
        }
    });
    if resources.camera_dynamic_capacity_slots < required_slots {
        let mut new_camera_slots = resources.camera_dynamic_capacity_slots.max(1);
        while new_camera_slots < required_slots {
            new_camera_slots *= 2;
        }
        let new_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Camera Dynamic Buffer"),
            size: resources.camera_dynamic_stride * new_camera_slots as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let new_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Light Storage Buffer"),
            size: (std::mem::size_of::<TwoDLightRaw>() * resources.light_capacity_slots.max(1))
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let library = render_state.library.as_ref().expect("library must exist");
        let new_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("2D Global BG"),
                layout: &resources.global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &new_camera_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<TwoDCameraRaw>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: new_light_buffer.as_entire_binding(),
                    },
                ],
            });
        resources.camera_dynamic_buffer = new_camera_buffer;
        resources.light_storage_buffer = new_light_buffer;
        resources.global_bind_group = new_bind_group;
        resources.camera_dynamic_capacity_slots = new_camera_slots;
    }
}

pub fn pass_2d_prepare(render_state: &mut RenderState) {
    let prepared = &mut render_state.two_d_prepared;
    prepared.cameras.clear();
    prepared.items.clear();

    prepared.cameras.extend(
        render_state
            .two_d_source
            .cameras
            .iter()
            .map(|(camera_id, record)| TwoDPreparedCamera {
                camera_id: *camera_id,
                transform: record.transform,
                near_far: record.near_far,
                ortho_scale: record.ortho_scale,
                layer_mask: record.layer_mask,
                order: record.order,
            }),
    );
    prepared.items.extend(
        render_state
            .two_d_source
            .sprites
            .iter()
            .map(|(item_id, record)| TwoDPreparedItem {
                item_id: *item_id,
                kind: TwoDItemKind::Sprite,
                transform: record.transform,
                geometry_id: record.geometry_id,
                material_id: record.material_id,
                layer: record.layer,
            }),
    );
    prepared.items.extend(
        render_state
            .two_d_source
            .shapes
            .iter()
            .map(|(item_id, record)| TwoDPreparedItem {
                item_id: *item_id,
                kind: TwoDItemKind::Shape,
                transform: record.transform,
                geometry_id: record.geometry_id,
                material_id: record.material_id,
                layer: record.layer,
            }),
    );

    prepared
        .cameras
        .sort_unstable_by_key(|camera| (camera.order, camera.camera_id));
    prepared.items.sort_unstable_by_key(|item| {
        (
            item.layer,
            match item.kind {
                TwoDItemKind::Sprite => 0_u8,
                TwoDItemKind::Shape => 1_u8,
            },
            item.item_id,
        )
    });
}

pub fn pass_2d_batch(render_state: &mut RenderState) {
    let batched = &mut render_state.two_d_batched;
    batched.items.clear();
    batched.ranges.clear();

    batched
        .items
        .extend(render_state.two_d_prepared.items.iter().cloned());
    batched.items.sort_unstable_by_key(|item| {
        (
            TwoDBatchKey {
                layer: item.layer,
                material_id: item
                    .material_id
                    .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
                geometry_id: item.geometry_id,
                kind: item.kind,
            },
            item.item_id,
        )
    });

    let mut i = 0usize;
    while i < batched.items.len() {
        let first = &batched.items[i];
        let key = TwoDBatchKey {
            layer: first.layer,
            material_id: first
                .material_id
                .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
            geometry_id: first.geometry_id,
            kind: first.kind,
        };
        let start = i;
        i += 1;
        while i < batched.items.len() {
            let item = &batched.items[i];
            let next_key = TwoDBatchKey {
                layer: item.layer,
                material_id: item
                    .material_id
                    .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
                geometry_id: item.geometry_id,
                kind: item.kind,
            };
            if next_key != key {
                break;
            }
            i += 1;
        }
        batched.ranges.push(TwoDBatchRange {
            key,
            start: start as u32,
            count: (i - start) as u32,
        });
    }
}

pub fn pass_2d_draw(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
) {
    const SHADER_ID_2D_BUILTIN: u64 = 0x0200_0001;
    if !render_state
        .material_shader_modules
        .contains_key(&SHADER_ID_2D_BUILTIN)
    {
        render_state.material_shader_modules.insert(
            SHADER_ID_2D_BUILTIN,
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("2D Builtin Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    r#"
struct CameraUniform {
    view_projection: mat4x4<f32>,
    tint: vec4<f32>,
    model_position: vec4<f32>,
    light_offset_count: vec4<u32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(2)
var linear_clamp_sampler: sampler;
@group(1) @binding(3)
var material_tex0: texture_2d<f32>;

struct Light2D {
    position: vec4<f32>,
    color: vec4<f32>,
    intensity_range: vec2<f32>,
    kind_flags: vec2<u32>,
}
@group(0) @binding(5)
var<storage, read> lights_2d: array<Light2D>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(4) uv0: vec2<f32>,
) -> VsOut {
    let world_pos = vec4<f32>(position, 1.0);
    var out: VsOut;
    out.position = camera.view_projection * world_pos;
    out.color = camera.tint;
    out.uv = uv0;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var lit = vec3<f32>(0.12, 0.12, 0.12);
    let light_count = min(camera.light_offset_count.y, 64u);
    for (var i: u32 = 0u; i < light_count; i = i + 1u) {
        let l = lights_2d[i];
        let to_light = l.position.xy - camera.model_position.xy;
        let dist = length(to_light);
        let range = max(l.intensity_range.y, 0.0001);
        let attenuation = max(1.0 - (dist / range), 0.0);
        lit += l.color.rgb * attenuation * l.intensity_range.x;
    }
    let base = textureSample(material_tex0, linear_clamp_sampler, in.uv) * in.color;
    return vec4<f32>(base.rgb * lit, base.a);
}
"#
                    .into(),
                ),
            }),
        );
    }
    let cameras = if render_state.two_d_prepared.cameras.is_empty() {
        vec![crate::core::render::state::TwoDPreparedCamera {
            camera_id: 0,
            transform: glam::Mat4::IDENTITY,
            near_far: glam::Vec2::new(0.0, 1.0),
            ortho_scale: 1.0,
            layer_mask: u32::MAX,
            order: 0,
        }]
    } else {
        render_state.two_d_prepared.cameras.clone()
    };
    let required_slots = (cameras.len() * (1 + render_state.two_d_batched.items.len())).max(1);
    ensure_two_d_pass_resources(render_state, device, queue, required_slots);
    let (
        pipeline_layout,
        camera_dynamic_buffer,
        light_storage_buffer,
        global_bind_group,
        camera_dynamic_stride,
        fallback_depth_view,
    ) = {
        let resources = render_state
            .two_d_pass_resources
            .as_ref()
            .expect("2D pass resources must be initialized");
        (
            resources.pipeline_layout.clone(),
            resources.camera_dynamic_buffer.clone(),
            resources.light_storage_buffer.clone(),
            resources.global_bind_group.clone(),
            resources.camera_dynamic_stride,
            resources.fallback_depth_view.clone(),
        )
    };
    let library = render_state.library.as_ref().expect("library must exist");
    let meta = TwoDFrameSemanticMeta {
        resolution: glam::Vec2::new(target_size.x.max(1) as f32, target_size.y.max(1) as f32),
        inv_resolution: glam::Vec2::new(
            1.0 / target_size.x.max(1) as f32,
            1.0 / target_size.y.max(1) as f32,
        ),
        frame_index: frame_index as u32,
        flags: 1,
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
                label: Some("2D Frame Semantics Buffer"),
                size: meta_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
    }
    let Some(frame_semantics_buffer) = render_state.forward_semantics_buffer.as_ref() else {
        return;
    };
    queue.write_buffer(frame_semantics_buffer, 0, meta_bytes);
    let frame_semantics_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("2D Frame Semantics BG"),
        layout: &library.layout_frame_semantics,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(target_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&fallback_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&library.fallback_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&library.fallback_view),
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
                resource: frame_semantics_buffer.as_entire_binding(),
            },
        ],
    });

    let draw_batches = {
        let scene = &render_state.scene;
        match render_state.vertex.as_mut() {
            Some(vertex_sys) => resolve_2d_draw_batches(
                &render_state.two_d_batched.ranges,
                |material_id| {
                    if material_id == crate::core::resources::MATERIAL_FALLBACK_ID {
                        return true;
                    }
                    scene
                        .materials
                        .get(&material_id)
                        .map(material_allows_2d)
                        .unwrap_or(false)
                },
                |geometry_id| matches!(vertex_sys.index_info(geometry_id), Ok(Some(index_info)) if index_info.count > 0),
            ),
            None => Vec::new(),
        }
    };
    let camera_visible_lights: Vec<Vec<TwoDLightRaw>> = cameras
        .iter()
        .map(|camera| collect_visible_2d_lights(render_state, camera))
        .collect();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("2D Draw Pass"),
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
    let mut current_pipeline_key: Option<PipelineKey> = None;

    // Pipeline/material binding is introduced in the next phase; for now we resolve valid batches
    // and consume the batched state deterministically inside the render pass.
    if let Some(vertex_sys) = render_state.vertex.as_mut() {
        vertex_sys.begin_pass();
        let mut camera_slot_index: usize = 0;
        for (camera_index, camera) in cameras.iter().enumerate() {
            let camera_vp = build_2d_view_projection(Some(camera), target_size);
            let visible_lights = &camera_visible_lights[camera_index];
            if !visible_lights.is_empty() {
                queue.write_buffer(
                    &light_storage_buffer,
                    0,
                    bytemuck::cast_slice(visible_lights),
                );
            }
            // Reserve one slot per camera to keep deterministic offset mapping and sizing.
            camera_slot_index = camera_slot_index.saturating_add(1);
            for batch in &draw_batches {
                vertex_sys.begin_pass();
                if !layer_visible_in_camera(batch.key.layer, camera.layer_mask) {
                    continue;
                }
                let Ok(Some(index_info)) = vertex_sys.index_info(batch.key.geometry_id) else {
                    continue;
                };
                if vertex_sys.bind(&mut pass, batch.key.geometry_id).is_err() {
                    continue;
                }
                let material = render_state
                    .scene
                    .materials
                    .get(&batch.key.material_id)
                    .or_else(|| {
                        render_state
                            .scene
                            .materials
                            .get(&crate::core::resources::MATERIAL_FALLBACK_ID)
                    });
                let surface_type = material
                    .map(|record| record.surface_type)
                    .unwrap_or(crate::core::resources::SurfaceType::Opaque);
                let topology = material
                    .map(|record| record.topology)
                    .unwrap_or(crate::core::resources::PrimitiveTopology::TriangleList);
                let polygon_mode = material
                    .map(|record| record.polygon_mode)
                    .unwrap_or(crate::core::resources::PolygonMode::Fill);
                let render_side = material
                    .map(|record| record.render_side)
                    .unwrap_or(crate::core::resources::RenderSide::Front);
                let blend = match surface_type {
                    crate::core::resources::SurfaceType::Transparent => {
                        Some(wgpu::BlendState::ALPHA_BLENDING)
                    }
                    crate::core::resources::SurfaceType::Opaque
                    | crate::core::resources::SurfaceType::Masked => None,
                };
                let cull_mode = match render_side {
                    crate::core::resources::RenderSide::Front => Some(wgpu::Face::Back),
                    crate::core::resources::RenderSide::Back => Some(wgpu::Face::Front),
                    crate::core::resources::RenderSide::DoubleSide => None,
                };
                let shader_id = if let Some(record) = material {
                    if material_uses_custom_2d_shader(record) {
                        if let Some(source) = record.compiled_shader_source.as_ref() {
                            let resolved_id = if record.compiled_shader_hash == 0 {
                                1
                            } else {
                                record.compiled_shader_hash
                            };
                            if !render_state
                                .material_shader_modules
                                .contains_key(&resolved_id)
                            {
                                render_state.material_shader_modules.insert(
                                    resolved_id,
                                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                        label: Some("2D Material Shader"),
                                        source: wgpu::ShaderSource::Wgsl(source.clone().into()),
                                    }),
                                );
                            }
                            resolved_id
                        } else {
                            SHADER_ID_2D_BUILTIN
                        }
                    } else {
                        SHADER_ID_2D_BUILTIN
                    }
                } else {
                    SHADER_ID_2D_BUILTIN
                };
                let Some(shader_module) = render_state.material_shader_modules.get(&shader_id)
                else {
                    continue;
                };
                let pipeline_key = PipelineKey {
                    shader_id,
                    color_format: target_format,
                    color_target_count: 1,
                    depth_format: None,
                    sample_count: 1,
                    topology: to_wgpu_topology(topology),
                    polygon_mode: to_wgpu_polygon_mode(polygon_mode),
                    cull_mode,
                    front_face: wgpu::FrontFace::Ccw,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    blend,
                };
                if current_pipeline_key != Some(pipeline_key) {
                    let pipeline =
                        render_state
                            .cache
                            .get_or_create(pipeline_key, frame_index, || {
                                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                    label: Some("2D Builtin Pipeline"),
                                    layout: Some(&pipeline_layout),
                                    vertex: wgpu::VertexState {
                                        module: shader_module,
                                        entry_point: Some("vs_main"),
                                        compilation_options:
                                            wgpu::PipelineCompilationOptions::default(),
                                        buffers: &[
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Position
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x3,
                                                    offset: 0,
                                                    shader_location: 0,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Normal
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Tangent
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Color0
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x4,
                                                    offset: 0,
                                                    shader_location: 3,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::UV0
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x2,
                                                    offset: 0,
                                                    shader_location: 4,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::UV1
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Joints
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Weights
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                        ],
                                    },
                                    fragment: Some(wgpu::FragmentState {
                                        module: shader_module,
                                        entry_point: Some("fs_main"),
                                        compilation_options:
                                            wgpu::PipelineCompilationOptions::default(),
                                        targets: &[Some(wgpu::ColorTargetState {
                                            format: target_format,
                                            blend,
                                            write_mask: wgpu::ColorWrites::ALL,
                                        })],
                                    }),
                                    primitive: wgpu::PrimitiveState {
                                        topology: to_wgpu_topology(topology),
                                        strip_index_format: None,
                                        front_face: wgpu::FrontFace::Ccw,
                                        cull_mode,
                                        unclipped_depth: false,
                                        polygon_mode: to_wgpu_polygon_mode(polygon_mode),
                                        conservative: false,
                                    },
                                    depth_stencil: None,
                                    multisample: wgpu::MultisampleState::default(),
                                    multiview_mask: None,
                                    cache: None,
                                })
                            });
                    pass.set_pipeline(pipeline);
                    current_pipeline_key = Some(pipeline_key);
                }
                let marker = format!(
                    "2d.camera={} layer={} material={} geometry={} kind={:?} start={} count={}",
                    camera.camera_id,
                    batch.key.layer,
                    batch.key.material_id,
                    batch.key.geometry_id,
                    batch.key.kind,
                    batch.start,
                    batch.count,
                );
                pass.insert_debug_marker(&marker);
                pass.set_bind_group(0, &global_bind_group, &[0]);
                pass.set_bind_group(2, &frame_semantics_bind_group, &[]);
                if let Some(material) = material {
                    if let Some(group) = material.bind_group.as_ref() {
                        if let Some(material_slot) = render_state
                            .material_uniform_slots
                            .get(&batch.key.material_id)
                            .copied()
                        {
                            let Some(bindings) = render_state.bindings.as_ref() else {
                                continue;
                            };
                            let material_offset =
                                bindings.material_3d_pool.get_offset(material_slot) as u32;
                            pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }
                }

                let start = batch.start as usize;
                let end = start.saturating_add(batch.count as usize);
                let Some(items) = render_state.two_d_batched.items.get(start..end) else {
                    continue;
                };
                if items.is_empty() {
                    continue;
                }
                let material_tint =
                    material_tint_for_batch(&render_state.scene, batch.key.material_id);
                for item in items {
                    let item_vp = camera_vp * item.transform;
                    let camera_raw = TwoDCameraRaw {
                        view_projection: item_vp,
                        tint: material_tint,
                        model_position: item.transform.w_axis,
                        light_offset_count: glam::UVec4::new(0, visible_lights.len() as u32, 0, 0),
                    };
                    let offset = (camera_slot_index as u64) * camera_dynamic_stride;
                    queue.write_buffer(
                        &camera_dynamic_buffer,
                        offset,
                        bytemuck::bytes_of(&camera_raw),
                    );
                    pass.set_bind_group(0, &global_bind_group, &[offset as u32]);
                    camera_slot_index = camera_slot_index.saturating_add(1);
                    pass.draw_indexed(0..index_info.count, 0, 0..1);
                }
            }
        }
    } else {
        for batch in &draw_batches {
            let marker = format!(
                "2d.batch layer={} material={} geometry={} kind={:?} start={} count={}",
                batch.key.layer,
                batch.key.material_id,
                batch.key.geometry_id,
                batch.key.kind,
                batch.start,
                batch.count,
            );
            pass.insert_debug_marker(&marker);
        }
    }
}

fn to_wgpu_topology(
    topology: crate::core::resources::PrimitiveTopology,
) -> wgpu::PrimitiveTopology {
    match topology {
        crate::core::resources::PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
        crate::core::resources::PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        crate::core::resources::PrimitiveTopology::TriangleList => {
            wgpu::PrimitiveTopology::TriangleList
        }
    }
}

fn to_wgpu_polygon_mode(mode: crate::core::resources::PolygonMode) -> wgpu::PolygonMode {
    match mode {
        crate::core::resources::PolygonMode::Fill => wgpu::PolygonMode::Fill,
        crate::core::resources::PolygonMode::Line => wgpu::PolygonMode::Line,
        crate::core::resources::PolygonMode::Point => wgpu::PolygonMode::Point,
    }
}

fn build_2d_view_projection(
    camera: Option<&crate::core::render::state::TwoDPreparedCamera>,
    target_size: glam::UVec2,
) -> glam::Mat4 {
    let width = target_size.x.max(1) as f32;
    let height = target_size.y.max(1) as f32;
    let aspect = width / height;
    match camera {
        Some(camera) => {
            let scale = camera.ortho_scale.max(1e-4);
            let half_h = scale;
            let half_w = half_h * aspect;
            let near = camera.near_far.x;
            let far = camera.near_far.y;
            let proj = glam::Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, near, far);
            let view = camera.transform.inverse();
            proj * view
        }
        None => glam::Mat4::orthographic_rh(-aspect, aspect, -1.0, 1.0, 0.0, 1.0),
    }
}

fn layer_visible_in_camera(layer: i32, layer_mask: u32) -> bool {
    if layer < 0 || layer > 31 {
        return false;
    }
    let bit = 1_u32 << (layer as u32);
    (layer_mask & bit) != 0
}

#[cfg(test)]
mod tests {
    use super::{
        layer_visible_in_camera, material_allows_2d, material_tint_for_batch,
        material_uses_custom_2d_shader, pass_2d_batch, pass_2d_prepare, resolve_2d_draw_batches,
    };
    use crate::core::render::RenderState;
    use crate::core::render::state::{TwoDBatchKey, TwoDBatchRange, TwoDItemKind};
    use crate::core::resources::{Camera2dRecord, Shape2dRecord, Sprite2dRecord};

    #[test]
    fn prepare_2d_collects_and_sorts_items_deterministically() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.cameras.insert(
            7,
            Camera2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                near_far: glam::Vec2::new(0.01, 10.0),
                ortho_scale: 2.0,
                layer_mask: 1,
                order: 2,
            },
        );
        render_state.two_d_source.cameras.insert(
            2,
            Camera2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                near_far: glam::Vec2::new(0.01, 10.0),
                ortho_scale: 1.0,
                layer_mask: 1,
                order: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            10,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 1,
                material_id: Some(100),
                layer: 4,
            },
        );
        render_state.two_d_source.shapes.insert(
            4,
            Shape2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 2,
                material_id: None,
                layer: 4,
            },
        );
        render_state.two_d_source.sprites.insert(
            3,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 3,
                material_id: None,
                layer: 1,
            },
        );

        pass_2d_prepare(&mut render_state);

        let camera_order: Vec<u32> = render_state
            .two_d_prepared
            .cameras
            .iter()
            .map(|camera| camera.camera_id)
            .collect();
        assert_eq!(camera_order, vec![2, 7]);

        let item_order: Vec<u32> = render_state
            .two_d_prepared
            .items
            .iter()
            .map(|item| item.item_id)
            .collect();
        assert_eq!(item_order, vec![3, 10, 4]);
    }

    #[test]
    fn batch_2d_groups_by_layer_material_geometry_and_kind() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.sprites.insert(
            10,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            11,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );
        render_state.two_d_source.shapes.insert(
            20,
            Shape2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            12,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 9,
                material_id: None,
                layer: 2,
            },
        );

        pass_2d_prepare(&mut render_state);
        pass_2d_batch(&mut render_state);

        assert_eq!(render_state.two_d_batched.ranges.len(), 3);
        assert_eq!(render_state.two_d_batched.ranges[0].count, 2);
        assert_eq!(render_state.two_d_batched.ranges[1].count, 1);
        assert_eq!(render_state.two_d_batched.ranges[2].count, 1);
        assert_eq!(render_state.two_d_batched.ranges[0].key.layer, 1);
        assert_eq!(render_state.two_d_batched.ranges[1].key.layer, 1);
        assert_eq!(render_state.two_d_batched.ranges[2].key.layer, 2);
    }

    #[test]
    fn batch_2d_keeps_deterministic_order_for_same_batch_key() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.sprites.insert(
            300,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            100,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            200,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
            },
        );

        pass_2d_prepare(&mut render_state);
        pass_2d_batch(&mut render_state);

        let item_order: Vec<u32> = render_state
            .two_d_batched
            .items
            .iter()
            .map(|item| item.item_id)
            .collect();
        assert_eq!(item_order, vec![100, 200, 300]);
        assert_eq!(render_state.two_d_batched.ranges.len(), 1);
        assert_eq!(render_state.two_d_batched.ranges[0].count, 3);
    }

    #[test]
    fn resolve_draw_batches_filters_missing_material_or_geometry() {
        let ranges = vec![
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 0,
                    material_id: 10,
                    geometry_id: 20,
                    kind: TwoDItemKind::Sprite,
                },
                start: 0,
                count: 2,
            },
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 0,
                    material_id: 11,
                    geometry_id: 21,
                    kind: TwoDItemKind::Shape,
                },
                start: 2,
                count: 3,
            },
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 1,
                    material_id: 12,
                    geometry_id: 22,
                    kind: TwoDItemKind::Sprite,
                },
                start: 5,
                count: 0,
            },
        ];
        let resolved = resolve_2d_draw_batches(
            &ranges,
            |material_id| material_id == 10,
            |geometry_id| geometry_id == 20,
        );
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].start, 0);
        assert_eq!(resolved[0].count, 2);
        assert_eq!(resolved[0].key.material_id, 10);
        assert_eq!(resolved[0].key.geometry_id, 20);
    }

    #[test]
    fn material_tint_uses_first_material_input_or_white() {
        let mut scene = crate::core::render::state::RenderScene::default();
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.inputs[0] = glam::Vec4::new(0.25, 0.5, 0.75, 1.0);
        scene.materials.insert(5, material);
        assert_eq!(
            material_tint_for_batch(&scene, 5),
            glam::Vec4::new(0.25, 0.5, 0.75, 1.0)
        );
        assert_eq!(material_tint_for_batch(&scene, 99), glam::Vec4::ONE);
    }

    #[test]
    fn material_tint_falls_back_to_white_when_alpha_is_zero() {
        let mut scene = crate::core::render::state::RenderScene::default();
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.inputs[0] = glam::Vec4::new(1.0, 0.0, 0.0, 0.0);
        scene.materials.insert(15, material);
        assert_eq!(material_tint_for_batch(&scene, 15), glam::Vec4::ONE);
    }

    #[test]
    fn material_custom_2d_shader_requires_realm_and_compiled_shader() {
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        assert!(!material_uses_custom_2d_shader(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_uses_custom_2d_shader(&material));
        material.compiled_shader_source = None;
        assert!(!material_uses_custom_2d_shader(&material));
        material.compiled_shader_source = Some("@vertex fn vs_main(){}".to_string());
        material.compile_error = None;
        assert!(material_uses_custom_2d_shader(&material));
        material.compile_error = Some("broken".to_string());
        assert!(!material_uses_custom_2d_shader(&material));
    }

    #[test]
    fn material_realm_kind_controls_2d_eligibility() {
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.realm_kind = crate::core::resources::MaterialRealmKind::ThreeD;
        assert!(!material_allows_2d(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_allows_2d(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_allows_2d(&material));
    }

    #[test]
    fn layer_visibility_respects_bit_mask_and_bounds() {
        let layer_mask = (1_u32 << 1) | (1_u32 << 4);
        assert!(layer_visible_in_camera(1, layer_mask));
        assert!(layer_visible_in_camera(4, layer_mask));
        assert!(!layer_visible_in_camera(0, layer_mask));
        assert!(!layer_visible_in_camera(-1, layer_mask));
        assert!(!layer_visible_in_camera(32, layer_mask));
    }
}

pub fn pass_2d_compose(
    _render_state: &mut RenderState,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
    _encoder: &mut wgpu::CommandEncoder,
    _target_view: &wgpu::TextureView,
    _target_format: wgpu::TextureFormat,
    _target_size: glam::UVec2,
    _frame_index: u64,
) {
    // 2D draw pass writes directly into the target view. For the default 2D graph,
    // compose is intentionally a no-op to avoid overwriting the frame with 3D compose.
}
