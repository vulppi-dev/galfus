use crate::core::image::ImagePixels;
use crate::core::realm::RealmId;
use crate::core::ui::events::{UiEvent, UiEventKind};
use crate::core::ui::state::{
    UiAnimKey, UiAnimProperty, UiAnimState, UiDocument, UiImageRecord, UiNodeEntry, UiState,
};
use crate::core::ui::types::{
    UiAlign, UiAnimEasing, UiAnimSpec, UiColor, UiImageSource, UiLayout, UiLayoutDirection,
    UiLength, UiNodeId, UiNodeProps, UiPadding, UiSize,
};
use std::hash::{Hash, Hasher};

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

pub fn render_realm_documents(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    realm_id: RealmId,
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

fn render_children(
    ui: &mut egui::Ui,
    document: &UiDocument,
    parent: Option<UiNodeId>,
    children: &[UiNodeId],
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
) {
    let ordered: Vec<UiNodeId> = match parent {
        Some(parent_id) => document
            .ordered_children
            .get(&parent_id)
            .cloned()
            .unwrap_or_else(|| children.to_vec()),
        None => {
            if document.ordered_root.is_empty() {
                children.to_vec()
            } else {
                document.ordered_root.clone()
            }
        }
    };

    for node_id in ordered {
        if let Some(entry) = document.nodes.get(&node_id) {
            render_node(
                ui,
                document,
                entry,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
            );
        }
    }
}

fn render_node(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
) {
    let display = entry.node.display.unwrap_or(true);
    if !display {
        return;
    }
    let visible = entry.node.visible.unwrap_or(true);
    let mut opacity = entry.node.opacity.unwrap_or(1.0);
    let mut translate_y = 0.0;
    if let Some(anim) = entry.node.anim.as_ref() {
        if let Some(spec) = anim.opacity {
            opacity = resolve_anim(
                ui_state,
                document.document_id,
                entry.node.id,
                UiAnimProperty::Opacity,
                spec,
                time_seconds,
                realm_id,
                ui_events,
            );
        }
        if let Some(spec) = anim.translate_y {
            translate_y = resolve_anim(
                ui_state,
                document.document_id,
                entry.node.id,
                UiAnimProperty::TranslateY,
                spec,
                time_seconds,
                realm_id,
                ui_events,
            );
        }
    }
    let opacity = opacity.clamp(0.0, 1.0);

    let debug = ui_state.debug;
    let response = ui.scope(|ui| {
        if !visible || opacity <= 0.0 {
            ui.set_invisible();
        }
        if opacity < 1.0 {
            ui.set_opacity(opacity);
        }
        if translate_y.abs() > f32::EPSILON {
            let mut rect = ui.available_rect_before_wrap();
            rect = rect.translate(egui::vec2(0.0, translate_y));
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                render_node_inner(ui, document, entry, ui_state, realm_id, ui_events, time_seconds);
            });
        } else {
            render_node_inner(ui, document, entry, ui_state, realm_id, ui_events, time_seconds);
        }
    });

    if debug.enabled && (debug.show_bounds || debug.show_ids) {
        let painter = ui.painter();
        let rect = response.response.rect;
        if debug.show_bounds {
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(0, 200, 255, 140)),
            );
        }
        if debug.show_ids {
            painter.text(
                rect.min,
                egui::Align2::LEFT_TOP,
                format!("{}", entry.node.id),
                egui::TextStyle::Monospace.resolve(ui.style()),
                egui::Color32::from_rgba_premultiplied(0, 200, 255, 200),
            );
        }
    }
}

fn render_node_inner(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
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
                        render_layout(
                            ui,
                            document,
                            entry,
                            layout,
                            ui_state,
                            realm_id,
                            ui_events,
                            time_seconds,
                        );
                    });
                } else {
                    render_layout(
                        ui,
                        document,
                        entry,
                        layout,
                        ui_state,
                        realm_id,
                        ui_events,
                        time_seconds,
                    );
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
            let response = ui.add_enabled(enabled, egui::Button::new(label.clone()));
            if response.clicked() {
                ui_events.push(UiEvent {
                    realm_id: realm_id.0,
                    document_id: document.document_id,
                    node_id: entry.node.id,
                    kind: UiEventKind::Click,
                    label: Some(label),
                });
            }
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
            let response = ui.add_enabled(enabled, edit);
            if response.changed() && response.lost_focus() {
                ui_events.push(UiEvent {
                    realm_id: realm_id.0,
                    document_id: document.document_id,
                    node_id: entry.node.id,
                    kind: UiEventKind::ChangeCommit,
                    label: Some(text),
                });
            }
        }
        UiNodeProps::Image { source, size } => match source {
            UiImageSource::UiImage(image_id) => {
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
            UiImageSource::Target(target_id) => {
                if let Some(target_size) = ui_state.external_textures.get(&target_id).copied() {
                    let size = resolve_size(size, target_size);
                    ui_state.target_size_requests.insert(
                        target_id,
                        glam::UVec2::new(
                            size.x.max(1.0).round() as u32,
                            size.y.max(1.0).round() as u32,
                        ),
                    );
                    let texture = egui::load::SizedTexture::new(
                        egui::TextureId::User(target_id),
                        size,
                    );
                    ui.add(egui::Image::from_texture(texture).fit_to_exact_size(size));
                } else {
                    ui.label("Target missing");
                }
            }
        },
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
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
) {
    let align = match layout.align {
        UiAlign::Start => egui::Align::Min,
        UiAlign::Center => egui::Align::Center,
        UiAlign::End => egui::Align::Max,
        UiAlign::Stretch => egui::Align::Min,
    };
    let justify = match layout.justify {
        UiAlign::Start => egui::Align::Min,
        UiAlign::Center => egui::Align::Center,
        UiAlign::End => egui::Align::Max,
        UiAlign::Stretch => egui::Align::Min,
    };

    let mut layout_def = match layout.direction {
        UiLayoutDirection::Row => egui::Layout::left_to_right(align),
        UiLayoutDirection::RowReverse => egui::Layout::right_to_left(align),
        UiLayoutDirection::Column => egui::Layout::top_down(align),
        UiLayoutDirection::ColumnReverse => egui::Layout::bottom_up(align),
        UiLayoutDirection::Grid => {
            let columns = layout.columns.unwrap_or(2).max(1);
            let grid_id = egui::Id::new(("grid", entry.node.id));
            egui::Grid::new(grid_id)
                .num_columns(columns as usize)
                .show(ui, |ui| {
                    render_children(
                        ui,
                        document,
                        Some(entry.node.id),
                        &entry.children,
                        ui_state,
                        realm_id,
                        ui_events,
                        time_seconds,
                    );
                });
            return;
        }
    };

    layout_def = layout_def.with_main_align(justify).with_main_wrap(layout.wrap);
    if layout.wrap {
        if let Some(limit) = layout.wrap_limit {
            if layout_def.is_horizontal() {
                ui.set_max_height(limit.max(0.0));
            } else {
                ui.set_max_width(limit.max(0.0));
            }
        }
    }

    ui.with_layout(layout_def, |ui| {
        render_children(
            ui,
            document,
            Some(entry.node.id),
            &entry.children,
            ui_state,
            realm_id,
            ui_events,
            time_seconds,
        );
    });
}

fn resolve_anim(
    ui_state: &mut UiState,
    document_id: u32,
    node_id: u32,
    property: UiAnimProperty,
    spec: UiAnimSpec,
    time_seconds: f64,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
) -> f32 {
    let key = UiAnimKey {
        document_id,
        node_id,
        property,
    };
    let duration = (spec.duration_ms as f32 / 1000.0).max(0.0001);
    let entry = ui_state.animations.entry(key).or_insert(UiAnimState {
        start_time: time_seconds,
        from: spec.from,
        to: spec.to,
        duration,
        finished: false,
        last_value: spec.from,
    });

    let elapsed = (time_seconds - entry.start_time).max(0.0) as f32;
    let t = (elapsed / entry.duration).clamp(0.0, 1.0);
    let eased = apply_easing(t, spec.easing);
    let value = entry.from + (entry.to - entry.from) * eased;
    entry.last_value = value;

    if t >= 1.0 && !entry.finished {
        entry.finished = true;
        ui_events.push(UiEvent {
            realm_id: realm_id.0,
            document_id,
            node_id,
            kind: UiEventKind::AnimComplete,
            label: Some(match property {
                UiAnimProperty::Opacity => "opacity".into(),
                UiAnimProperty::TranslateY => "translateY".into(),
            }),
        });
    }

    value
}

fn apply_easing(value: f32, easing: UiAnimEasing) -> f32 {
    match easing {
        UiAnimEasing::Linear => value,
        UiAnimEasing::EaseInOut => {
            if value < 0.5 {
                2.0 * value * value
            } else {
                -1.0 + (4.0 - 2.0 * value) * value
            }
        }
    }
}

fn apply_theme(ctx: &egui::Context, theme: &crate::core::ui::state::UiThemeState) {
    let mut style = (*ctx.style()).clone();
    let mut text_color: Option<egui::Color32> = None;
    let mut panel_fill: Option<egui::Color32> = None;
    let mut window_fill: Option<egui::Color32> = None;
    let mut accent: Option<egui::Color32> = None;
    let mut font_size: Option<f32> = None;

    for (key, value) in &theme.data {
        match (key.as_str(), value) {
            ("fontSize", crate::core::ui::types::UiThemeValue::Float(v)) => {
                font_size = Some(*v as f32);
            }
            ("textColor", crate::core::ui::types::UiThemeValue::String(v)) => {
                text_color = parse_color_string(v);
            }
            ("panelFill", crate::core::ui::types::UiThemeValue::String(v)) => {
                panel_fill = parse_color_string(v);
            }
            ("windowFill", crate::core::ui::types::UiThemeValue::String(v)) => {
                window_fill = parse_color_string(v);
            }
            ("accentColor", crate::core::ui::types::UiThemeValue::String(v)) => {
                accent = parse_color_string(v);
            }
            _ => {}
        }
    }

    if let Some(size) = font_size {
        for text_style in style.text_styles.values_mut() {
            text_style.size = size;
        }
    }
    if let Some(color) = text_color {
        style.visuals.override_text_color = Some(color);
    }
    if let Some(color) = panel_fill {
        style.visuals.panel_fill = color;
    }
    if let Some(color) = window_fill {
        style.visuals.window_fill = color;
    }
    if let Some(color) = accent {
        style.visuals.selection.bg_fill = color;
    }

    ctx.set_style(style);
}

fn parse_color_string(value: &str) -> Option<egui::Color32> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        let parsed = match hex.len() {
            6 => u32::from_str_radix(hex, 16).ok().map(|v| (v, 255u8)),
            8 => u32::from_str_radix(hex, 16).ok().map(|v| (v >> 8, (v & 0xFF) as u8)),
            _ => None,
        };
        if let Some((rgb, a)) = parsed {
            let r = ((rgb >> 16) & 0xFF) as u8;
            let g = ((rgb >> 8) & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;
            return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
        }
    }

    let parts: Vec<_> = trimmed.split(',').map(|p| p.trim()).collect();
    if parts.len() >= 3 {
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        let a = parts
            .get(3)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(255);
        return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
    }

    None
}

pub(crate) fn hash_shapes(shapes: &[egui::epaint::ClippedShape]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    shapes.len().hash(&mut hasher);
    for shape in shapes {
        format!("{:?}", shape).hash(&mut hasher);
    }
    hasher.finish()
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
