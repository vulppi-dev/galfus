use crate::core::render::RenderState;
use crate::core::render::state::ResourceLibrary;

mod overlay;
pub use overlay::pass_compose_surface;

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
    let Some(camera_id) = render_state.camera_order.first().copied() else {
        return;
    };
    let Some(record) = render_state.camera_record(camera_id) else {
        return;
    };
    let source = record
        .post_target
        .as_ref()
        .or(record.render_target.as_ref());
    let Some(source_target) = source else {
        return;
    };
    let source_view = source_target.view.clone();
    let source_size = glam::UVec2::new(
        source_target.texture.size().width,
        source_target.texture.size().height,
    );

    pass_compose_surface(
        render_state,
        device,
        queue,
        encoder,
        target_view,
        target_format,
        glam::UVec2::new(target_width.max(1), target_height.max(1)),
        &source_view,
        source_size,
        frame_index,
    );
}
