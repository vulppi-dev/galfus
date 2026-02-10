use crate::core::realm::RealmId;
use crate::core::render::RenderState;
use crate::core::ui::render::{render_realm_documents, sync_ui_images};
use crate::core::ui::UiState;

pub fn pass_ui(
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    realm_id: RealmId,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
) {
    ui_state.ensure_realm(realm_id);
    let (context, pixels_per_point) = {
        let Some(ui_realm) = ui_state.realm_mut(realm_id) else {
            return;
        };
        ui_realm.last_frame_index = frame_index;
        (ui_realm.context.clone(), ui_realm.pixels_per_point)
    };

    let screen_size = egui::vec2(target_size.x as f32, target_size.y as f32);
    let mut input = egui::RawInput::default();
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);
    input.screen_rect = Some(screen_rect);
    if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport.native_pixels_per_point = Some(pixels_per_point);
        viewport.inner_rect = Some(screen_rect);
        viewport.outer_rect = Some(screen_rect);
        viewport.focused = Some(true);
    }
    input.time = Some(frame_index as f64);
    sync_ui_images(&context, ui_state);
    let output = context.run(input, |ctx| {
        render_realm_documents(ctx, ui_state, realm_id);
    });

    let clipped_primitives = context.tessellate(output.shapes, output.pixels_per_point);

    let renderer = render_state
        .ui_renderer
        .get_or_insert_with(|| crate::core::ui::UiRenderer::new(device, queue, target_format));
    renderer.update_textures(device, queue, &output.textures_delta);
    renderer.render(
        device,
        queue,
        encoder,
        target_view,
        target_format,
        target_size,
        output.pixels_per_point,
        &clipped_primitives,
    );
}
