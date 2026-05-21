#![allow(dead_code)]

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::render::state::SampledTargetBindKey;

use super::build_compose_bind_group;

pub struct ComposeOverlay<'a> {
    pub source_view: &'a wgpu::TextureView,
    pub source_size: glam::UVec2,
    pub rect: glam::Vec4,
    pub clip: Option<glam::Vec4>,
    pub blend: Option<wgpu::BlendState>,
    pub opacity: f32,
}

pub fn pass_compose_overlays(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    overlays: &[ComposeOverlay<'_>],
    frame_index: u64,
) {
    if overlays.is_empty() {
        return;
    }

    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };
    let uniform_buffer = match render_state.compose_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Overlay Pass"),
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
    let mut used_bind_keys = std::collections::HashSet::new();
    let mut compose_bind_cache_hits = 0u32;
    let mut compose_bind_cache_misses = 0u32;

    for overlay in overlays {
        let (viewport, scissor) = match resolve_overlay_geometry(overlay, target_size) {
            Some(result) => result,
            None => continue,
        };

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
            blend: overlay.blend,
        };

        let pipeline = render_state.cache.get_or_create(key, frame_index, || {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compose Overlay Pipeline Layout"),
                bind_group_layouts: &[&library.layout_target],
                ..Default::default()
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Compose Overlay Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &library.compose_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &library.compose_shader,
                    entry_point: Some("fs_overlay"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: overlay.blend,
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

        let bind_key = SampledTargetBindKey {
            target_view_ptr: overlay.source_view as *const wgpu::TextureView as usize,
            outline_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
            ssao_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
            bloom_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
            uniform_buffer_ptr: uniform_buffer as *const wgpu::Buffer as usize,
        };
        used_bind_keys.insert(bind_key);
        if render_state.compose_bind_cache.contains_key(&bind_key) {
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
                    overlay.source_view,
                    &library.fallback_view,
                    &library.fallback_view,
                    &library.fallback_view,
                    uniform_buffer,
                )
            });
        let opacity_uniform = [overlay.opacity.clamp(0.0, 1.0), 0.0, 0.0, 0.0];
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&opacity_uniform));

        render_pass.set_pipeline(pipeline);
        render_pass.set_viewport(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(scissor.x, scissor.y, scissor.width, scissor.height);
        render_pass.set_bind_group(0, &*bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
    drop(render_pass);
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

pub fn pass_compose_surface(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    source_view: &wgpu::TextureView,
    source_size: glam::UVec2,
    frame_index: u64,
) {
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };
    let uniform_buffer = match render_state.compose_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

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

    let pipeline = render_state.cache.get_or_create(key, frame_index, || {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compose Surface Pipeline Layout"),
            bind_group_layouts: &[&library.layout_target],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Compose Surface Pipeline"),
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

    let target_w = target_size.x.max(1) as f32;
    let target_h = target_size.y.max(1) as f32;
    let source_w = source_size.x.max(1) as f32;
    let source_h = source_size.y.max(1) as f32;
    let target_aspect = target_w / target_h;
    let source_aspect = source_w / source_h;

    let (scale_x, scale_y) = if target_aspect > source_aspect {
        (1.0, (source_aspect / target_aspect).clamp(0.0, 1.0))
    } else {
        ((target_aspect / source_aspect).clamp(0.0, 1.0), 1.0)
    };
    let offset_x = (1.0 - scale_x) * 0.5;
    let offset_y = (1.0 - scale_y) * 0.5;
    let cover = [scale_x, scale_y, offset_x, offset_y];
    queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&cover));

    let bind_key = SampledTargetBindKey {
        target_view_ptr: source_view as *const wgpu::TextureView as usize,
        outline_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
        ssao_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
        bloom_view_ptr: &library.fallback_view as *const wgpu::TextureView as usize,
        uniform_buffer_ptr: uniform_buffer as *const wgpu::Buffer as usize,
    };
    if render_state.compose_bind_cache.contains_key(&bind_key) {
        render_state.compose_bind_cache_hits =
            render_state.compose_bind_cache_hits.saturating_add(1);
    } else {
        render_state.compose_bind_cache_misses =
            render_state.compose_bind_cache_misses.saturating_add(1);
    }
    let bind_group = render_state
        .compose_bind_cache
        .entry(bind_key)
        .or_insert_with(|| {
            build_compose_bind_group(
                device,
                library,
                source_view,
                &library.fallback_view,
                &library.fallback_view,
                &library.fallback_view,
                uniform_buffer,
            )
        });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Surface Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
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

    render_pass.set_pipeline(pipeline);
    render_pass.set_viewport(
        0.0,
        0.0,
        target_size.x as f32,
        target_size.y as f32,
        0.0,
        1.0,
    );
    render_pass.set_bind_group(0, &*bind_group, &[]);
    render_pass.draw(0..3, 0..1);
    drop(render_pass);
    render_state
        .compose_bind_cache
        .retain(|key, _| *key == bind_key);
}

#[derive(Debug, Clone, Copy)]
struct OverlayViewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy)]
struct OverlayScissor {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn resolve_overlay_geometry(
    overlay: &ComposeOverlay<'_>,
    target_size: glam::UVec2,
) -> Option<(OverlayViewport, OverlayScissor)> {
    let rect = overlay.rect;
    if rect.z <= 0.0 || rect.w <= 0.0 {
        return None;
    }

    let source_width = overlay.source_size.x.max(1) as f32;
    let source_height = overlay.source_size.y.max(1) as f32;
    let scale = rect.w / source_height;
    let draw_width = (source_width * scale).max(1.0);

    // CSS-like cover anchored by center: keep height fitted to rect.h and clip sides.
    let mut viewport_x = rect.x + (rect.z - draw_width) * 0.5;
    let mut viewport_y = rect.y;
    let mut viewport_width = draw_width;
    let mut viewport_height = rect.w.max(1.0);

    if viewport_x < 0.0 {
        viewport_width = (viewport_width + viewport_x).max(0.0);
        viewport_x = 0.0;
    }
    if viewport_y < 0.0 {
        viewport_height = (viewport_height + viewport_y).max(0.0);
        viewport_y = 0.0;
    }

    let max_width = target_size.x as f32 - viewport_x;
    let max_height = target_size.y as f32 - viewport_y;
    if max_width <= 0.0 || max_height <= 0.0 {
        return None;
    }
    viewport_width = viewport_width.min(max_width);
    viewport_height = viewport_height.min(max_height);

    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return None;
    }

    let mut clip_rect = rect;
    if let Some(clip) = overlay.clip {
        clip_rect = intersect_rect(clip_rect, clip);
    }

    let scissor = rect_to_scissor(clip_rect, target_size)?;

    Some((
        OverlayViewport {
            x: viewport_x,
            y: viewport_y,
            width: viewport_width,
            height: viewport_height,
        },
        scissor,
    ))
}

fn intersect_rect(a: glam::Vec4, b: glam::Vec4) -> glam::Vec4 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.z).min(b.x + b.z);
    let y2 = (a.y + a.w).min(b.y + b.w);
    glam::Vec4::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}

fn rect_to_scissor(rect: glam::Vec4, target_size: glam::UVec2) -> Option<OverlayScissor> {
    let mut x = rect.x.floor();
    let mut y = rect.y.floor();
    let mut width = rect.z.ceil();
    let mut height = rect.w.ceil();

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    if x < 0.0 {
        width = (width + x).max(0.0);
        x = 0.0;
    }
    if y < 0.0 {
        height = (height + y).max(0.0);
        y = 0.0;
    }

    let max_width = target_size.x as f32 - x;
    let max_height = target_size.y as f32 - y;
    if max_width <= 0.0 || max_height <= 0.0 {
        return None;
    }

    width = width.min(max_width);
    height = height.min(max_height);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(OverlayScissor {
        x: x as u32,
        y: y as u32,
        width: width as u32,
        height: height as u32,
    })
}
