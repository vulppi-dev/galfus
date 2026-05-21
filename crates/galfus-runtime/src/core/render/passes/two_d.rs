use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::render::state::{
    TwoDBatchKey, TwoDBatchRange, TwoDItemKind, TwoDPreparedCamera, TwoDPreparedItem,
    TwoDTextureBindKey,
};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDInstanceRaw {
    model_col0: [f32; 4],
    model_col1: [f32; 4],
    model_col2: [f32; 4],
    model_col3: [f32; 4],
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDCameraRaw {
    view_projection: [[f32; 4]; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoDDrawBatch {
    key: TwoDBatchKey,
    start: u32,
    count: u32,
}

const CAP_REALM_2D: &str = "realm:2d";
const CAP_LAYOUT_2D_V1: &str = "layout:2d-v1";

fn material_allows_2d(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    matches!(
        record.realm_kind,
        crate::core::resources::MaterialRealmKind::TwoD
            | crate::core::resources::MaterialRealmKind::Both
    )
}

fn material_supports_2d_layout(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    let has_realm_2d = record
        .shader_capabilities
        .iter()
        .any(|capability| capability == CAP_REALM_2D);
    let has_layout_2d_v1 = record
        .shader_capabilities
        .iter()
        .any(|capability| capability == CAP_LAYOUT_2D_V1);
    has_realm_2d && has_layout_2d_v1
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

fn material_base_texture_id(
    scene: &crate::core::render::state::RenderScene,
    material_id: u32,
) -> Option<u32> {
    let material = scene.materials.get(&material_id)?;
    let texture_id = material.texture_ids[0];
    if texture_id == crate::core::resources::SHADER_MATERIAL_INVALID_SLOT {
        return None;
    }
    Some(texture_id)
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
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var base_tex: texture_2d<f32>;
@group(1) @binding(1)
var base_sampler: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(3) _color: vec4<f32>,
    @location(4) uv0: vec2<f32>,
    @location(8) model_col0: vec4<f32>,
    @location(9) model_col1: vec4<f32>,
    @location(10) model_col2: vec4<f32>,
    @location(11) model_col3: vec4<f32>,
    @location(12) tint: vec4<f32>,
) -> VsOut {
    let model = mat4x4<f32>(model_col0, model_col1, model_col2, model_col3);
    let world_pos = model * vec4<f32>(position, 1.0);
    var out: VsOut;
    out.position = camera.view_projection * world_pos;
    out.color = tint;
    out.uv = uv0;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(base_tex, base_sampler, in.uv);
    return in.color * texel;
}
"#
                    .into(),
                ),
            }),
        );
    }
    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2D Camera BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2D Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    let fallback_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("2D Fallback White Texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &fallback_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[255, 255, 255, 255],
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
    let fallback_tex_view = fallback_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("2D Builtin Pipeline Layout"),
        bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
        ..Default::default()
    });
    let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("2D Camera Buffer"),
        size: std::mem::size_of::<TwoDCameraRaw>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("2D Camera BG"),
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
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

        for camera in &cameras {
            let camera_vp = build_2d_view_projection(Some(camera), target_size);
            let camera_raw = TwoDCameraRaw {
                view_projection: camera_vp.to_cols_array_2d(),
            };
            queue.write_buffer(&camera_buffer, 0, bytemuck::bytes_of(&camera_raw));
            pass.set_bind_group(0, &camera_bind_group, &[]);
            let base_sampler = render_state
                .library
                .as_ref()
                .map(|library| library.samplers.linear_clamp.clone());

            for batch in &draw_batches {
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
                    if material_supports_2d_layout(record) {
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
                                                array_stride: std::mem::size_of::<TwoDInstanceRaw>()
                                                    as u64,
                                                step_mode: wgpu::VertexStepMode::Instance,
                                                attributes: &[
                                                    wgpu::VertexAttribute {
                                                        format: wgpu::VertexFormat::Float32x4,
                                                        offset: 0,
                                                        shader_location: 8,
                                                    },
                                                    wgpu::VertexAttribute {
                                                        format: wgpu::VertexFormat::Float32x4,
                                                        offset: 16,
                                                        shader_location: 9,
                                                    },
                                                    wgpu::VertexAttribute {
                                                        format: wgpu::VertexFormat::Float32x4,
                                                        offset: 32,
                                                        shader_location: 10,
                                                    },
                                                    wgpu::VertexAttribute {
                                                        format: wgpu::VertexFormat::Float32x4,
                                                        offset: 48,
                                                        shader_location: 11,
                                                    },
                                                    wgpu::VertexAttribute {
                                                        format: wgpu::VertexFormat::Float32x4,
                                                        offset: 64,
                                                        shader_location: 12,
                                                    },
                                                ],
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
                let texture_view =
                    material_base_texture_id(&render_state.scene, batch.key.material_id)
                        .and_then(|texture_id| render_state.scene.textures.get(&texture_id))
                        .map(|record| &record.view)
                        .unwrap_or(&fallback_tex_view);
                let Some(base_sampler) = base_sampler.as_ref() else {
                    continue;
                };
                let bind_key = TwoDTextureBindKey {
                    texture_view_ptr: texture_view as *const _ as usize,
                    sampler_ptr: base_sampler as *const _ as usize,
                };
                let texture_bind_group = render_state
                    .two_d_texture_bind_cache
                    .entry(bind_key)
                    .or_insert_with(|| {
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("2D Texture BG"),
                            layout: &texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(base_sampler),
                                },
                            ],
                        })
                    });
                let texture_bind_group_ref: &wgpu::BindGroup = texture_bind_group;
                pass.set_bind_group(1, texture_bind_group_ref, &[]);

                let start = batch.start as usize;
                let end = start.saturating_add(batch.count as usize);
                let Some(items) = render_state.two_d_batched.items.get(start..end) else {
                    continue;
                };
                if items.is_empty() {
                    continue;
                }
                let tint = material_tint_for_batch(&render_state.scene, batch.key.material_id);
                let instance_data: Vec<TwoDInstanceRaw> = items
                    .iter()
                    .map(|item| {
                        let cols = item.transform.to_cols_array_2d();
                        TwoDInstanceRaw {
                            model_col0: cols[0],
                            model_col1: cols[1],
                            model_col2: cols[2],
                            model_col3: cols[3],
                            tint: tint.to_array(),
                        }
                    })
                    .collect();
                let instance_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("2D Instance Buffer"),
                        contents: bytemuck::cast_slice(instance_data.as_slice()),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                pass.set_vertex_buffer(5, instance_buffer.slice(..));
                pass.draw_indexed(0..index_info.count, 0, 0..instance_data.len() as u32);
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
        CAP_LAYOUT_2D_V1, CAP_REALM_2D, material_supports_2d_layout, material_tint_for_batch,
        pass_2d_batch, pass_2d_prepare, resolve_2d_draw_batches,
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
    fn material_layout_2d_support_requires_realm_and_layout_caps() {
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        assert!(!material_supports_2d_layout(&material));
        material.shader_capabilities.push(CAP_REALM_2D.to_string());
        assert!(!material_supports_2d_layout(&material));
        material
            .shader_capabilities
            .push(CAP_LAYOUT_2D_V1.to_string());
        assert!(material_supports_2d_layout(&material));
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
