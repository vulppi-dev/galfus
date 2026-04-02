mod image_utils;
mod node_helpers;
mod node_render;
mod node_render_controls;
mod node_render_layout;
mod node_render_media;
mod theme;

use crate::core::image::ImagePixels;
use crate::core::realm::RealmId;
use crate::core::ui::events::{UiEvent, UiEventKind};
use crate::core::ui::state::{
    UiAnimKey, UiAnimProperty, UiAnimState, UiDocument, UiImageRecord, UiNodeEntry, UiState,
};
use crate::core::ui::types::{
    UiAlign, UiAnimEasing, UiAnimSpec, UiColor, UiImageSource, UiLayout, UiLayoutDirection,
    UiLength, UiNodeId, UiNodeProps, UiPadding, UiPanelKind, UiSize, UiSplitDirection, UiStroke,
};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use image_utils::{collect_live_ui_image_ids, image_buffer_to_color_image};
use node_render::render_children;
use theme::apply_theme;

pub fn sync_ui_images(ctx: &egui::Context, ui_state: &mut UiState) {
    let live_image_ids = collect_live_ui_image_ids(ui_state);
    for (image_id, record) in ui_state.images.iter_mut() {
        if !live_image_ids.contains(image_id) {
            record.texture = None;
            continue;
        }
        if record.texture.is_some() {
            continue;
        }
        let color_image = image_buffer_to_color_image(record);
        let handle = ctx.load_texture(
            format!("ui_image_{}", image_id),
            color_image,
            egui::TextureOptions::LINEAR,
        );
        record.texture = Some(handle);
    }
}

pub fn render_realm_documents(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    realm_id: RealmId,
    realm_size: glam::UVec2,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
) {
    let mut document_ids: Vec<_> = ui_state
        .documents
        .values()
        .filter(|doc| doc.realm_id == realm_id)
        .map(|doc| doc.document_id)
        .collect();
    document_ids.sort();

    for document_id in document_ids {
        let document = {
            let Some(doc) = ui_state.documents.get_mut(&document_id) else {
                continue;
            };
            doc.ensure_layout_cache();
            if let Some(theme_id) = doc.theme_id {
                if let Some(theme) = ui_state.themes.get(&theme_id) {
                    if doc.last_theme_version != Some(theme.version) {
                        apply_theme(ctx, theme);
                        doc.last_theme_version = Some(theme.version);
                    }
                }
            }
            doc.clone()
        };
        let rect = resolve_document_rect(document.rect, realm_size);
        let area_id = egui::Id::new((realm_id.0, document.document_id));
        egui::Area::new(area_id)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                ui.set_min_size(rect.size());
                ui.set_max_size(rect.size());
                ui.set_clip_rect(rect);
                render_children(
                    ui,
                    &document,
                    None,
                    &document.root_children,
                    ui_state,
                    realm_id,
                    ui_events,
                    time_seconds,
                );
            });
    }
}

pub(crate) fn hash_shapes(shapes: &[egui::epaint::ClippedShape]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    shapes.len().hash(&mut hasher);
    for shape in shapes {
        format!("{:?}", shape).hash(&mut hasher);
    }
    hasher.finish()
}

fn resolve_document_rect(rect: glam::Vec4, realm_size: glam::UVec2) -> egui::Rect {
    let max_w = realm_size.x.max(1) as f32;
    let max_h = realm_size.y.max(1) as f32;
    let x = rect.x.max(0.0).min(max_w);
    let y = rect.y.max(0.0).min(max_h);
    let mut w = rect.z;
    let mut h = rect.w;
    if w <= 0.0 {
        w = (max_w - x).max(1.0);
    }
    if h <= 0.0 {
        h = (max_h - y).max(1.0);
    }
    let clamped_w = w.max(1.0).min((max_w - x).max(1.0));
    let clamped_h = h.max(1.0).min((max_h - y).max(1.0));
    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(clamped_w, clamped_h))
}
