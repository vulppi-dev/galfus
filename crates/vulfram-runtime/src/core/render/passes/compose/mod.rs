use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::render::state::ResourceLibrary;
use crate::core::render::state::SampledTargetBindKey;

mod overlay;
pub use overlay::{ComposeOverlay, pass_compose_overlays, pass_compose_surface};

pub(super) fn build_compose_bind_group(
    device: &wgpu::Device,
    library: &ResourceLibrary,
    target_view: &wgpu::TextureView,
    outline_view: &wgpu::TextureView,
    ssao_view: &wgpu::TextureView,
    bloom_view: &wgpu::TextureView,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compose Bind Group"),
        layout: &library.layout_target,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(target_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(outline_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(ssao_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(bloom_view),
            },
        ],
    })
}

pub fn pass_compose_to_view(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_width: u32,
    target_height: u32,
    frame_index: u64,
) {
    // 2. Get or Create Compose Pipeline
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let uniform_buffer = match render_state.compose_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    // 1. Sort cameras by order

    let cache = &mut render_state.cache;
    let key = PipelineKey {
        shader_id: ShaderId::Compose as u64,
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

    let pipeline = cache.get_or_create(key, frame_index, || {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compose Pipeline Layout"),
            bind_group_layouts: &[&library.layout_target],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Compose Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &library.compose_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &library.compose_shader,
                entry_point: Some("fs_cover"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: key.blend,
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

    // 3. Begin compose pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Pass"),
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

    render_pass.set_pipeline(pipeline);

    let mut used_bind_keys = std::collections::HashSet::new();
    let mut compose_bind_cache_hits = 0u32;
    let mut compose_bind_cache_misses = 0u32;
    for camera_id in render_state.camera_order.iter().copied() {
        let Some(record) = render_state.scene.cameras.get(&camera_id) else {
            continue;
        };
        let target = match record
            .post_target
            .as_ref()
            .or(record.render_target.as_ref())
        {
            Some(t) => t,
            None => continue,
        };
        let outline_view = record
            .outline_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let ssao_view = record
            .ssao_blur_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let bloom_view = record
            .bloom_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);

        // 4. Resolve viewport
        let (x, y) = record
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_position(target_width, target_height))
            .unwrap_or((0, 0));

        let (width, height) = record
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_size(target_width, target_height))
            .unwrap_or((target_width, target_height));

        let viewport_width = width.max(1) as f32;
        let viewport_height = height.max(1) as f32;
        let source_size = target.texture.size();
        let source_width = source_size.width.max(1) as f32;
        let source_height = source_size.height.max(1) as f32;
        let viewport_aspect = viewport_width / viewport_height;
        let source_aspect = source_width / source_height;
        let (scale_x, scale_y) = if viewport_aspect > source_aspect {
            (1.0, (source_aspect / viewport_aspect).clamp(0.0, 1.0))
        } else {
            ((viewport_aspect / source_aspect).clamp(0.0, 1.0), 1.0)
        };
        let cover = [
            scale_x,
            scale_y,
            (1.0 - scale_x) * 0.5,
            (1.0 - scale_y) * 0.5,
        ];
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&cover));

        render_pass.set_viewport(x as f32, y as f32, width as f32, height as f32, 0.0, 1.0);

        let bind_key = SampledTargetBindKey {
            target_view_ptr: &target.view as *const wgpu::TextureView as usize,
            outline_view_ptr: outline_view as *const wgpu::TextureView as usize,
            ssao_view_ptr: ssao_view as *const wgpu::TextureView as usize,
            bloom_view_ptr: bloom_view as *const wgpu::TextureView as usize,
            uniform_buffer_ptr: uniform_buffer as *const wgpu::Buffer as usize,
        };
        used_bind_keys.insert(bind_key);
        let bind_cache_hit = render_state.compose_bind_cache.contains_key(&bind_key);
        if bind_cache_hit {
            compose_bind_cache_hits = compose_bind_cache_hits.saturating_add(1);
        } else {
            compose_bind_cache_misses = compose_bind_cache_misses.saturating_add(1);
        }
        let bind_group = render_state
            .compose_bind_cache
            .entry(bind_key)
            .or_insert_with(|| {
                build_compose_bind_group(
                    device,
                    library,
                    &target.view,
                    outline_view,
                    ssao_view,
                    bloom_view,
                    uniform_buffer,
                )
            });

        render_pass.set_bind_group(0, &*bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
    render_state
        .compose_bind_cache
        .retain(|key, _| used_bind_keys.contains(key));
    render_state.compose_bind_cache_hits = render_state
        .compose_bind_cache_hits
        .saturating_add(compose_bind_cache_hits);
    render_state.compose_bind_cache_misses = render_state
        .compose_bind_cache_misses
        .saturating_add(compose_bind_cache_misses);
}
