use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceTable};
use crate::core::resources::RenderTarget;
use crate::core::target::{TargetId, TargetKind, TargetTable};
use crate::core::render::RenderState;
use crate::core::ui::events::UiEvent;
use crate::core::ui::render::{render_realm_documents, sync_ui_images};
use crate::core::ui::renderer::ExternalTextureInput;
use crate::core::ui::UiState;
use std::collections::HashMap;

pub fn pass_ui(
    render_state: &mut RenderState,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    targets: &TargetTable,
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
) {
    ui_state.ensure_realm(realm_id);
    let external_inputs =
        collect_external_textures(ui_state, targets, surfaces, auto_links, surface_targets, realm_id);
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
    input.time = Some(frame_index as f64);
    input.events = input_events;
    input.modifiers = modifiers;
    sync_ui_images(&context, ui_state);
    let output = context.run(input, |ctx| {
        render_realm_documents(ctx, ui_state, realm_id, ui_events);
    });

    let clipped_primitives = context.tessellate(output.shapes, output.pixels_per_point);

    let renderer = render_state
        .ui_renderer
        .get_or_insert_with(|| crate::core::ui::UiRenderer::new(device, queue, target_format));
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
}

fn collect_external_textures<'a>(
    ui_state: &mut UiState,
    targets: &TargetTable,
    surfaces: &SurfaceTable,
    auto_links: &HashMap<(u32, TargetId), AutoLink>,
    surface_targets: &'a HashMap<SurfaceId, RenderTarget>,
    realm_id: RealmId,
) -> Vec<ExternalTextureInput<'a>> {
    ui_state.external_textures.clear();
    let mut target_surfaces: HashMap<TargetId, SurfaceId> = HashMap::new();

    for ((link_realm, target_id), link) in auto_links.iter() {
        let Some(target) = targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Texture {
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
            view: &surface_target.view,
            size,
            source_ptr: surface_target as *const RenderTarget as usize,
        });
    }

    inputs
}
