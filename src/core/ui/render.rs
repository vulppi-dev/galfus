use crate::core::image::ImagePixels;
use crate::core::realm::RealmId;
use crate::core::ui::state::{UiDocument, UiImageRecord, UiNodeEntry, UiState};
use crate::core::ui::types::{
    UiAlign, UiColor, UiLayout, UiLayoutDirection, UiLength, UiNodeId, UiNodeProps, UiPadding,
    UiSize,
};

pub fn sync_ui_images(ctx: &egui::Context, ui_state: &mut UiState) {
    for (image_id, record) in ui_state.images.iter_mut() {
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

pub fn render_realm_documents(ctx: &egui::Context, ui_state: &UiState, realm_id: RealmId) {
    let mut documents: Vec<&UiDocument> = ui_state
        .documents
        .values()
        .filter(|doc| doc.realm_id == realm_id)
        .collect();
    documents.sort_by_key(|doc| doc.document_id);

    for document in documents {
        let rect = egui::Rect::from_min_size(
            egui::pos2(document.rect.x, document.rect.y),
            egui::vec2(document.rect.z, document.rect.w),
        );
        let area_id = egui::Id::new((realm_id.0, document.document_id));
        egui::Area::new(area_id)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                ui.set_min_size(rect.size());
                ui.set_max_size(rect.size());
                ui.set_clip_rect(rect);
                render_children(ui, document, &document.root_children, ui_state);
            });
    }
}

fn render_children(
    ui: &mut egui::Ui,
    document: &UiDocument,
    children: &[UiNodeId],
    ui_state: &UiState,
) {
    for node_id in children {
        if let Some(entry) = document.nodes.get(node_id) {
            render_node(ui, document, entry, ui_state);
        }
    }
}

fn render_node(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &UiState,
) {
    match entry.node.props.clone() {
        UiNodeProps::Container {
            layout,
            padding,
            size,
            scroll_x,
            scroll_y,
        } => {
            let frame = build_padding_frame(padding);
            frame.show(ui, |ui| {
                apply_size(ui, size);
                let spacing = ui.spacing().item_spacing;
                if layout.gap > 0.0 {
                    ui.spacing_mut().item_spacing =
                        egui::vec2(layout.gap, layout.gap);
                }
                if scroll_x || scroll_y {
                    let mut scroll = egui::ScrollArea::new([scroll_x, scroll_y]);
                    scroll = scroll.auto_shrink([false, false]);
                    scroll.show(ui, |ui| {
                        render_layout(ui, document, entry, layout, ui_state);
                    });
                } else {
                    render_layout(ui, document, entry, layout, ui_state);
                }
                ui.spacing_mut().item_spacing = spacing;
            });
        }
        UiNodeProps::Text { text, size, color } => {
            let mut rich = egui::RichText::new(text);
            if let Some(size) = size {
                rich = rich.size(size);
            }
            if let Some(color) = color {
                rich = rich.color(color_to_color32(color));
            }
            ui.label(rich);
        }
        UiNodeProps::Button { label, enabled } => {
            let enabled = enabled.unwrap_or(true);
            ui.add_enabled(enabled, egui::Button::new(label));
        }
        UiNodeProps::Input {
            value,
            placeholder,
            enabled,
        } => {
            let mut text = value.clone();
            let mut edit = egui::TextEdit::singleline(&mut text);
            if let Some(placeholder) = placeholder {
                edit = edit.hint_text(placeholder);
            }
            let enabled = enabled.unwrap_or(true);
            ui.add_enabled(enabled, edit);
        }
        UiNodeProps::Image { image_id, size } => {
            if let Some(record) = ui_state.images.get(&image_id) {
                if let Some(texture) = &record.texture {
                    let size = resolve_size(size, record.size);
                    ui.add(egui::Image::new(texture).fit_to_exact_size(size));
                } else {
                    ui.label("Image pending");
                }
            } else {
                ui.label("Image missing");
            }
        }
        UiNodeProps::Separator => {
            ui.separator();
        }
        UiNodeProps::Spacer { width, height } => {
            let width = width.unwrap_or(0.0);
            let height = height.unwrap_or(0.0);
            if width > 0.0 || height > 0.0 {
                ui.add_space(width.max(height));
            } else {
                ui.add_space(0.0);
            }
        }
    }
}

fn render_layout(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    layout: UiLayout,
    ui_state: &UiState,
) {
    let align = match layout.align {
        UiAlign::Start => egui::Align::Min,
        UiAlign::Center => egui::Align::Center,
        UiAlign::End => egui::Align::Max,
        UiAlign::Stretch => egui::Align::Min,
    };

    match layout.direction {
        UiLayoutDirection::Row => {
            ui.with_layout(egui::Layout::left_to_right(align), |ui| {
                render_children(ui, document, &entry.children, ui_state);
            });
        }
        UiLayoutDirection::Column => {
            ui.with_layout(egui::Layout::top_down(align), |ui| {
                render_children(ui, document, &entry.children, ui_state);
            });
        }
        UiLayoutDirection::Grid => {
            let columns = layout.columns.unwrap_or(2).max(1);
            let grid_id = egui::Id::new(("grid", entry.node.id));
            egui::Grid::new(grid_id)
                .num_columns(columns as usize)
                .show(ui, |ui| {
                    render_children(ui, document, &entry.children, ui_state);
                });
        }
    }
}

fn apply_size(ui: &mut egui::Ui, size: Option<UiSize>) {
    let Some(size) = size else { return };
    match size.width {
        UiLength::Fill => ui.set_width(ui.available_width()),
        UiLength::Px(value) => ui.set_min_width(value.max(0.0)),
        UiLength::Auto => {}
    }
    match size.height {
        UiLength::Fill => ui.set_height(ui.available_height()),
        UiLength::Px(value) => ui.set_min_height(value.max(0.0)),
        UiLength::Auto => {}
    }
}

fn build_padding_frame(padding: Option<UiPadding>) -> egui::Frame {
    let padding = padding.unwrap_or(UiPadding {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    });
    egui::Frame::none().inner_margin(egui::Margin {
        left: padding.left,
        right: padding.right,
        top: padding.top,
        bottom: padding.bottom,
    })
}

fn resolve_size(size: Option<UiSize>, fallback: [u32; 2]) -> egui::Vec2 {
    let fallback = egui::vec2(fallback[0] as f32, fallback[1] as f32);
    let Some(size) = size else { return fallback };

    let width = match size.width {
        UiLength::Auto => fallback.x,
        UiLength::Fill => fallback.x,
        UiLength::Px(value) => value,
    };
    let height = match size.height {
        UiLength::Auto => fallback.y,
        UiLength::Fill => fallback.y,
        UiLength::Px(value) => value,
    };
    egui::vec2(width.max(0.0), height.max(0.0))
}

fn color_to_color32(color: UiColor) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

fn image_buffer_to_color_image(record: &UiImageRecord) -> egui::ColorImage {
    let size = [record.image.width as usize, record.image.height as usize];
    let mut pixels = Vec::with_capacity(size[0] * size[1]);

    match &record.image.pixels {
        ImagePixels::Rgba8(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                pixels.push(egui::Color32::from_rgba_unmultiplied(
                    chunk[0], chunk[1], chunk[2], chunk[3],
                ));
            }
        }
        ImagePixels::Rgba16F(bytes) => {
            for chunk in bytes.chunks_exact(4) {
                let r = half::f16::from_bits(chunk[0]).to_f32();
                let g = half::f16::from_bits(chunk[1]).to_f32();
                let b = half::f16::from_bits(chunk[2]).to_f32();
                let a = half::f16::from_bits(chunk[3]).to_f32();
                pixels.push(egui::Color32::from_rgba_unmultiplied(
                    to_u8(r),
                    to_u8(g),
                    to_u8(b),
                    to_u8(a),
                ));
            }
        }
    }

    egui::ColorImage { size, pixels }
}

fn to_u8(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0).round() as u8
}
