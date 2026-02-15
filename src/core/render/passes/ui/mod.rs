use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::render::RenderState;
use crate::core::resources::RenderTarget;
use crate::core::target::{TargetId, TargetKind, TargetLayerTable, TargetTable};
use crate::core::ui::UiState;
use crate::core::ui::events::UiEvent;
use crate::core::ui::render::{hash_shapes, render_realm_documents, sync_ui_images};
use crate::core::ui::renderer::ExternalTextureInput;
use std::collections::HashMap;
use std::time::Instant;

pub fn pass_ui(
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    realm_id: RealmId,
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
) {
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
            return;
        };
        ui_realm.last_frame_index = frame_index;
        (
            ui_realm.context.clone(),
            ui_realm.pixels_per_point,
            ui_realm.drain_events(),
            ui_realm.modifiers,
        )
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
    input.time = Some(time_seconds);
    input.events = input_events;
    input.modifiers = modifiers;
    sync_ui_images(&context, ui_state);
    let layout_start = Instant::now();
    let output = context.run(input, |ctx| {
        render_realm_documents(ctx, ui_state, realm_id, ui_events, time_seconds);
    });
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
    let clipped_primitives =
        cached.unwrap_or_else(|| context.tessellate(output.shapes, output.pixels_per_point));
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

    let debug = ui_state.debug;
    if let Some(realm) = ui_state.realm_mut(realm_id) {
        if debug.enabled && debug.show_profile {
            let painter = realm.context.debug_painter();
            let text = format!(
                "UI layout: {:.2}ms\nUI tess: {:.2}ms",
                realm.profile.layout_ms, realm.profile.tessellate_ms
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
}

fn collect_external_textures(
    render_state: &RenderState,
    ui_state: &mut UiState,
    targets: &TargetTable,
    target_layers: &TargetLayerTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> Vec<ExternalTextureInput> {
    ui_state.external_textures.clear();
    let mut target_surfaces: HashMap<TargetId, SurfaceId> = HashMap::new();
    let first_camera_id = first_camera_id(render_state);

    for ((link_realm, target_id), link) in auto_links.iter() {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Texture && target.kind != TargetKind::WidgetRealmViewport {
            continue;
        }

        match target_surfaces.entry(*target_id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(link.surface_id);
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if *link_realm == realm_id.0 {
                    entry.insert(link.surface_id);
                }
            }
        }
    }

    let mut inputs = Vec::new();

    for (target_id, surface_id) in target_surfaces {
        if let Some(target_state) = targets.entries.get(&target_id) {
            if target_state.kind == TargetKind::WidgetRealmViewport {
                let camera_id = target_layers
                    .entries
                    .iter()
                    .find_map(|((layer_realm, layer_target), layer)| {
                        if *layer_target == target_id {
                            if *layer_realm == realm_id.0 {
                                return layer.camera_id;
                            }
                            if layer.camera_id.is_some() {
                                return layer.camera_id;
                            }
                        }
                        None
                    })
                    .or(first_camera_id);
                if let Some(camera_id) = camera_id {
                    if let Some(camera) = render_state.scene.cameras.get(&camera_id) {
                        if let Some(camera_target) = camera.render_target.as_ref() {
                            let texture_size = camera_target._texture.size();
                            let size = [texture_size.width.max(1), texture_size.height.max(1)];
                            ui_state.external_textures.insert(target_id.0, size);
                            inputs.push(ExternalTextureInput {
                                id: target_id.0,
                                view: camera_target.view.clone(),
                                size,
                                source_ptr: camera_target as *const RenderTarget as usize,
                            });
                            continue;
                        }
                    }
                }
            }
        }

        let Some(surface_state) = surfaces.entries.get(&surface_id) else {
            continue;
        };
        let Some(surface_target) = surface_targets.get(&surface_id) else {
            continue;
        };
        let size = surface_state.value.size;
        let size = [size.x.max(1), size.y.max(1)];
        ui_state.external_textures.insert(target_id.0, size);

        inputs.push(ExternalTextureInput {
            id: target_id.0,
            view: surface_target.view.clone(),
            size,
            source_ptr: surface_target as *const RenderTarget as usize,
        });
    }

    inputs
}

fn first_camera_id(render_state: &RenderState) -> Option<u32> {
    if let Some(camera_id) = render_state.camera_order.first().copied() {
        return Some(camera_id);
    }
    let mut keys: Vec<u32> = render_state.scene.cameras.keys().copied().collect();
    keys.sort_unstable();
    keys.first().copied()
}
