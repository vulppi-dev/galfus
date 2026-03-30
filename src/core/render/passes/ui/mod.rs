mod external_textures;

pub use vulfram_render::UiPlatformAction;

use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::render::RenderState;
use crate::core::resources::RenderTarget;
use crate::core::target::{TargetId, TargetLayerTable, TargetTable};
use crate::core::time::Instant;
use crate::core::ui::UiState;
use crate::core::ui::events::UiEvent;
use crate::core::ui::render::{hash_shapes, render_realm_documents, sync_ui_images};
use std::collections::HashMap;
use std::sync::Arc;

use external_textures::collect_external_textures;

pub fn pass_ui(
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    realm_id: RealmId,
    window_id: u32,
    window_focused: bool,
    ui_events: &mut Vec<UiEvent>,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    time_seconds: f64,
) -> Vec<UiPlatformAction> {
    let has_documents_for_realm = ui_state
        .documents
        .values()
        .any(|document| document.realm_id == realm_id);
    if !has_documents_for_realm {
        return Vec::new();
    }

    ui_state.ensure_realm(realm_id);
    let external_inputs = collect_external_textures(
        render_state,
        ui_state,
        targets,
        target_layers,
        surfaces,
        auto_links,
        surface_targets,
        realm_id,
    );
    let (context, pixels_per_point, input_events, modifiers) = {
        let Some(ui_realm) = ui_state.realm_mut(realm_id) else {
            return Vec::new();
        };
        ui_realm.last_frame_index = frame_index;
        (
            ui_realm.context.clone(),
            ui_realm.pixels_per_point,
            ui_realm.drain_events(),
            ui_realm.modifiers,
        )
    };

    let safe_pixels_per_point = pixels_per_point.max(0.001);
    let screen_size = egui::vec2(
        target_size.x as f32 / safe_pixels_per_point,
        target_size.y as f32 / safe_pixels_per_point,
    );
    let mut input = egui::RawInput::default();
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);
    input.screen_rect = Some(screen_rect);
    input.focused = window_focused;
    if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport.native_pixels_per_point = Some(pixels_per_point);
        viewport.inner_rect = Some(screen_rect);
        viewport.outer_rect = Some(screen_rect);
        viewport.focused = Some(window_focused);
    }
    input.time = Some(time_seconds);
    input.events = input_events;
    input.modifiers = modifiers;
    sync_ui_images(&context, ui_state);
    let layout_start = Instant::now();
    let output = context.run(input, |ctx| {
        render_realm_documents(
            ctx,
            ui_state,
            realm_id,
            target_size,
            ui_events,
            time_seconds,
        );
    });
    let needs_repaint = context.has_requested_repaint();
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.needs_repaint = needs_repaint;
    }
    let platform_actions = vulfram_render::collect_platform_actions(&output, window_id, realm_id.0);
    let layout_ms = layout_start.elapsed().as_secs_f32() * 1000.0;

    let tess_start = Instant::now();
    let shapes_hash = hash_shapes(&output.shapes);
    let cached = ui_state
        .realm_mut(realm_id)
        .and_then(|realm| realm.tessellation_cache.as_ref())
        .filter(|cache| {
            cache.shapes_hash == shapes_hash
                && cache.pixels_per_point == output.pixels_per_point
                && output.textures_delta.set.is_empty()
        })
        .map(|cache| cache.clipped.clone());
    let clipped_primitives: Arc<[egui::ClippedPrimitive]> = cached
        .unwrap_or_else(|| Arc::from(context.tessellate(output.shapes, output.pixels_per_point)));
    let tess_ms = tess_start.elapsed().as_secs_f32() * 1000.0;
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.profile.layout_ms = layout_ms;
        realm.profile.tessellate_ms = tess_ms;
        realm.tessellation_cache = Some(crate::core::ui::state::UiTessellationCache {
            shapes_hash,
            pixels_per_point: output.pixels_per_point,
            clipped: clipped_primitives.clone(),
        });
    }

    let renderer = render_state
        .ui_renderers
        .entry(realm_id)
        .or_insert_with(|| crate::core::ui::UiRenderer::new(device, queue, target_format));
    renderer.update_textures(device, queue, &output.textures_delta);
    renderer.update_external_textures(device, &external_inputs);
    let render_stats = renderer.render(
        device,
        queue,
        encoder,
        target_view,
        target_format,
        target_size,
        output.pixels_per_point,
        clipped_primitives.as_ref(),
    );
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        realm.profile.upload_ms = render_stats.upload_ms;
        realm.profile.draw_ms = render_stats.draw_ms;
    }

    let debug = ui_state.debug;
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        if debug.enabled && debug.show_profile {
            let painter = realm.context.debug_painter();
            let text = format!(
                "UI input: {:.2}ms\nUI layout: {:.2}ms\nUI tess: {:.2}ms\nUI upload: {:.2}ms\nUI draw: {:.2}ms",
                realm.profile.input_routing_ms,
                realm.profile.layout_ms,
                realm.profile.tessellate_ms,
                realm.profile.upload_ms,
                realm.profile.draw_ms
            );
            painter.text(
                egui::pos2(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                text,
                egui::TextStyle::Monospace.resolve(&realm.context.style()),
                egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
            );
        }
    }
    platform_actions
}
