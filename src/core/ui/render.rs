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

fn collect_live_ui_image_ids(ui_state: &UiState) -> HashSet<u32> {
    let mut ids: HashSet<u32> = HashSet::new();
    for document in ui_state.documents.values() {
        for entry in document.nodes.values() {
            match &entry.node.props {
                UiNodeProps::Image {
                    source: UiImageSource::UiImage(image_id),
                    ..
                }
                | UiNodeProps::ImageButton {
                    source: UiImageSource::UiImage(image_id),
                    ..
                } => {
                    ids.insert(*image_id);
                }
                _ => {}
            }
        }
    }
    ids
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
                render_node_inner(
                    ui,
                    document,
                    entry,
                    ui_state,
                    realm_id,
                    ui_events,
                    time_seconds,
                );
            });
        } else {
            render_node_inner(
                ui,
                document,
                entry,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
            );
        }
    });

    if debug.enabled && (debug.show_bounds || debug.show_ids) {
        let painter = ui.painter();
        let rect = response.response.rect;
        if debug.show_bounds {
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(0, 200, 255, 140),
                ),
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

    let rect = response.response.rect;
    ui_state.layout_rects.insert(
        (document.document_id, entry.node.id),
        glam::vec4(rect.min.x, rect.min.y, rect.width(), rect.height()),
    );
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
    let node_tooltip = entry.node.tooltip.as_deref();
    let node_context_menu = entry.node.context_menu.as_deref();
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
                    ui.spacing_mut().item_spacing = egui::vec2(layout.gap, layout.gap);
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
        UiNodeProps::Window {
            title,
            open,
            movable,
            resizable,
            collapsible,
            anchored,
            size,
        } => {
            let open_key = (document.document_id, entry.node.id);
            let mut is_open = ui_state
                .node_open_state
                .get(&open_key)
                .copied()
                .unwrap_or(open.unwrap_or(true));
            let mut window = egui::Window::new(title).id(egui::Id::new(open_key));
            window = window
                .movable(movable.unwrap_or(true))
                .resizable(resizable.unwrap_or(true))
                .collapsible(collapsible.unwrap_or(true));
            if let Some(anchor) = anchored {
                window = window.anchor(
                    egui::Align2::LEFT_TOP,
                    egui::vec2(anchor.x.max(0.0), anchor.y.max(0.0)),
                );
            }
            if let Some(size) = size {
                let initial = resolve_size(Some(size), [320, 180]);
                window = window.default_size(initial);
            }
            window.open(&mut is_open).show(ui.ctx(), |ui| {
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
            ui_state.node_open_state.insert(open_key, is_open);
        }
        UiNodeProps::Panel {
            kind,
            resizable,
            size,
            min_size,
            max_size,
        } => {
            let panel_id = egui::Id::new((document.document_id, entry.node.id, "panel"));
            let fallback = resolve_size(size, [240, 180]);
            match kind {
                UiPanelKind::SideLeft => {
                    let mut panel =
                        egui::SidePanel::left(panel_id).resizable(resizable.unwrap_or(true));
                    panel = panel.default_width(fallback.x);
                    if let Some(min_size) = min_size {
                        panel = panel.min_width(min_size.max(0.0));
                    }
                    if let Some(max_size) = max_size {
                        panel = panel.max_width(max_size.max(0.0));
                    }
                    panel.show_inside(ui, |ui| {
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
                UiPanelKind::SideRight => {
                    let mut panel =
                        egui::SidePanel::right(panel_id).resizable(resizable.unwrap_or(true));
                    panel = panel.default_width(fallback.x);
                    if let Some(min_size) = min_size {
                        panel = panel.min_width(min_size.max(0.0));
                    }
                    if let Some(max_size) = max_size {
                        panel = panel.max_width(max_size.max(0.0));
                    }
                    panel.show_inside(ui, |ui| {
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
                UiPanelKind::Top => {
                    let mut panel =
                        egui::TopBottomPanel::top(panel_id).resizable(resizable.unwrap_or(true));
                    panel = panel.default_height(fallback.y);
                    if let Some(min_size) = min_size {
                        panel = panel.min_height(min_size.max(0.0));
                    }
                    if let Some(max_size) = max_size {
                        panel = panel.max_height(max_size.max(0.0));
                    }
                    panel.show_inside(ui, |ui| {
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
                UiPanelKind::Bottom => {
                    let mut panel =
                        egui::TopBottomPanel::bottom(panel_id).resizable(resizable.unwrap_or(true));
                    panel = panel.default_height(fallback.y);
                    if let Some(min_size) = min_size {
                        panel = panel.min_height(min_size.max(0.0));
                    }
                    if let Some(max_size) = max_size {
                        panel = panel.max_height(max_size.max(0.0));
                    }
                    panel.show_inside(ui, |ui| {
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
                UiPanelKind::Central => {
                    egui::CentralPanel::default().show_inside(ui, |ui| {
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
            }
        }
        UiNodeProps::SplitPane {
            direction,
            ratio,
            resizable,
            min_a,
            max_a,
            min_b,
            max_b,
        } => {
            render_split_pane(
                ui,
                document,
                entry,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
                direction,
                ratio,
                resizable.unwrap_or(true),
                min_a.unwrap_or(80.0),
                max_a,
                min_b.unwrap_or(80.0),
                max_b,
            );
        }
        UiNodeProps::Area {
            label,
            x,
            y,
            draggable,
            size,
        } => {
            let area_key = (document.document_id, entry.node.id);
            let mut position = ui_state
                .area_positions
                .get(&area_key)
                .copied()
                .unwrap_or_else(|| glam::Vec2::new(x.unwrap_or(0.0), y.unwrap_or(0.0)));
            let area_label = label.unwrap_or_else(|| format!("area-{}", entry.node.id));
            egui::Area::new(egui::Id::new((area_key, area_label.as_str())))
                .movable(draggable.unwrap_or(false))
                .fixed_pos(egui::pos2(position.x, position.y))
                .show(ui.ctx(), |ui| {
                    apply_size(ui, size);
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
                    let min = ui.min_rect().min;
                    position = glam::Vec2::new(min.x, min.y);
                });
            ui_state.area_positions.insert(area_key, position);
        }
        UiNodeProps::Frame {
            padding,
            fill,
            stroke,
            rounding,
            size,
        } => {
            let frame = build_custom_frame(padding, fill, stroke, rounding);
            frame.show(ui, |ui| {
                apply_size(ui, size);
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
        UiNodeProps::ScrollArea {
            scroll_x,
            scroll_y,
            auto_shrink,
            size,
        } => {
            apply_size(ui, size);
            let mut scroll = egui::ScrollArea::new([scroll_x, scroll_y]);
            if let Some(auto_shrink) = auto_shrink {
                scroll = scroll.auto_shrink([auto_shrink, auto_shrink]);
            }
            scroll.show(ui, |ui| {
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
        UiNodeProps::Grid {
            columns,
            striped,
            min_col_width,
            size,
        } => {
            apply_size(ui, size);
            let mut grid = egui::Grid::new(egui::Id::new((
                document.document_id,
                entry.node.id,
                "grid-v2",
            )))
            .num_columns(columns.unwrap_or(2).max(1) as usize);
            if let Some(striped) = striped {
                grid = grid.striped(striped);
            }
            if let Some(min_col_width) = min_col_width {
                grid = grid.min_col_width(min_col_width.max(0.0));
            }
            grid.show(ui, |ui| {
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
        UiNodeProps::Popup { title, open, size } => {
            let open_key = (document.document_id, entry.node.id);
            let mut state = ui_state
                .node_open_state
                .get(&open_key)
                .copied()
                .unwrap_or(open.unwrap_or(false));
            if state {
                let mut window = egui::Window::new(title.unwrap_or_else(|| "Popup".into()))
                    .id(egui::Id::new((open_key, "popup")))
                    .collapsible(false)
                    .movable(true)
                    .resizable(false)
                    .title_bar(true);
                if let Some(size) = size {
                    window = window.default_size(resolve_size(Some(size), [280, 160]));
                }
                window.open(&mut state).show(ui.ctx(), |ui| {
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
            ui_state.node_open_state.insert(open_key, state);
        }
        UiNodeProps::Tooltip { text } => {
            ui.label(text);
        }
        UiNodeProps::Modal { title, open, size } => {
            let open_key = (document.document_id, entry.node.id);
            let mut state = ui_state
                .node_open_state
                .get(&open_key)
                .copied()
                .unwrap_or(open.unwrap_or(false));
            if state {
                let screen_rect = ui.ctx().screen_rect();
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 160),
                );
                let mut window = egui::Window::new(title)
                    .id(egui::Id::new((open_key, "modal")))
                    .collapsible(false)
                    .movable(false)
                    .resizable(false)
                    .title_bar(true)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO);
                if let Some(size) = size {
                    window = window.default_size(resolve_size(Some(size), [360, 220]));
                }
                window.open(&mut state).show(ui.ctx(), |ui| {
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
            ui_state.node_open_state.insert(open_key, state);
        }
        UiNodeProps::Resize {
            size,
            min_size,
            max_size,
        } => {
            let mut resize = egui::Resize::default();
            if let Some(size) = size {
                resize = resize.default_size(resolve_size(Some(size), [240, 180]));
            }
            if let Some(size) = min_size {
                let resolved = resolve_size(Some(size), [16, 16]);
                resize = resize.min_size(resolved);
            }
            if let Some(size) = max_size {
                let resolved = resolve_size(Some(size), [2000, 2000]);
                resize = resize.max_size(resolved);
            }
            resize.show(ui, |ui| {
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
        UiNodeProps::Scene {
            size,
            zoom_min,
            zoom_max,
            pan_enabled,
        } => {
            render_scene_node(
                ui,
                document,
                entry,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
                size,
                zoom_min.unwrap_or(0.25),
                zoom_max.unwrap_or(4.0),
                pan_enabled.unwrap_or(true),
            );
        }
        UiNodeProps::Canvas { ops, size, clip } => {
            let fallback = resolve_size_in_ui(ui, size, [320, 180]);
            let (rect, _) = ui.allocate_exact_size(fallback, egui::Sense::hover());
            crate::core::ui::paint::paint_ops(ui, rect, &ops, clip.unwrap_or(true));
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
        UiNodeProps::RichText {
            text,
            size,
            color,
            strong,
            italics,
            underline,
            strikethrough,
            monospace,
        } => {
            let mut rich = egui::RichText::new(text);
            if let Some(size) = size {
                rich = rich.size(size);
            }
            if let Some(color) = color {
                rich = rich.color(color_to_color32(color));
            }
            if strong.unwrap_or(false) {
                rich = rich.strong();
            }
            if italics.unwrap_or(false) {
                rich = rich.italics();
            }
            if underline.unwrap_or(false) {
                rich = rich.underline();
            }
            if strikethrough.unwrap_or(false) {
                rich = rich.strikethrough();
            }
            if monospace.unwrap_or(false) {
                rich = rich.monospace();
            }
            ui.label(rich);
        }
        UiNodeProps::Link { label, enabled } => {
            let response = ui.add_enabled(enabled.unwrap_or(true), egui::Link::new(label.clone()));
            emit_interaction_events(
                response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
        }
        UiNodeProps::Hyperlink {
            label,
            url,
            enabled,
        } => {
            let response = if enabled.unwrap_or(true) {
                ui.hyperlink_to(label.clone(), url)
            } else {
                ui.add_enabled(false, egui::Link::new(label.clone()))
            };
            emit_interaction_events(
                response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
        }
        UiNodeProps::Button { label, enabled } => {
            let enabled = enabled.unwrap_or(true);
            let response = ui.add_enabled(enabled, egui::Button::new(label.clone()));
            emit_interaction_events(
                response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
        }
        UiNodeProps::Checkbox {
            label,
            checked,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let value = ui_state.bool_values.entry(key).or_insert(checked);
            let response = ui.add_enabled(
                enabled.unwrap_or(true),
                egui::Checkbox::new(value, label.clone()),
            );
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(value.to_string()),
                );
            }
        }
        UiNodeProps::Radio {
            label,
            selected,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let value = ui_state.bool_values.entry(key).or_insert(selected);
            let response = ui.add_enabled(
                enabled.unwrap_or(true),
                egui::RadioButton::new(*value, label.clone()),
            );
            if response.clicked() {
                *value = true;
            }
            emit_interaction_events(
                response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
        }
        UiNodeProps::SelectableLabel {
            label,
            selected,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let value = ui_state.bool_values.entry(key).or_insert(selected);
            let response = ui.add_enabled(
                enabled.unwrap_or(true),
                egui::SelectableLabel::new(*value, label.clone()),
            );
            if response.clicked() {
                *value = !*value;
            }
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(value.to_string()),
                );
            }
        }
        UiNodeProps::Toggle {
            label,
            value,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let state = ui_state.bool_values.entry(key).or_insert(value);
            let response = ui.add_enabled(
                enabled.unwrap_or(true),
                egui::Checkbox::new(state, label.clone()),
            );
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label.clone()),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(state.to_string()),
                );
            }
        }
        UiNodeProps::Slider {
            value,
            min,
            max,
            step,
            label,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let current = ui_state.number_values.entry(key).or_insert(value);
            let mut slider =
                egui::Slider::new(current, min..=max).step_by(step.unwrap_or(0.0).max(0.0));
            if let Some(label) = label.clone() {
                slider = slider.text(label);
            }
            let response = ui.add_enabled(enabled.unwrap_or(true), slider);
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                label,
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(current.to_string()),
                );
            }
            if response.drag_stopped() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::ChangeCommit,
                    Some(current.to_string()),
                );
            }
        }
        UiNodeProps::DragValue {
            value,
            speed,
            min,
            max,
            prefix,
            suffix,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let current = ui_state.number_values.entry(key).or_insert(value);
            let mut widget = egui::DragValue::new(current).speed(speed.unwrap_or(0.1));
            if let Some(min) = min {
                widget = widget.range(min..=max.unwrap_or(f64::MAX));
            } else if let Some(max) = max {
                widget = widget.range(f64::MIN..=max);
            }
            if let Some(prefix) = prefix {
                widget = widget.prefix(prefix);
            }
            if let Some(suffix) = suffix {
                widget = widget.suffix(suffix);
            }
            let response = ui.add_enabled(enabled.unwrap_or(true), widget);
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                None,
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(current.to_string()),
                );
            }
            if response.lost_focus() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::ChangeCommit,
                    Some(current.to_string()),
                );
            }
        }
        UiNodeProps::ProgressBar {
            value,
            text,
            animate,
            show_percentage,
        } => {
            let mut bar = egui::ProgressBar::new(value.clamp(0.0, 1.0) as f32);
            if let Some(text) = text {
                bar = bar.text(text);
            } else if show_percentage.unwrap_or(true) {
                bar = bar.show_percentage();
            }
            if animate.unwrap_or(false) {
                bar = bar.animate(true);
            }
            ui.add(bar);
        }
        UiNodeProps::ComboBox {
            label,
            selected,
            options,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let current = ui_state
                .selection_values
                .entry(key)
                .or_insert(selected.clone());
            let mut changed = false;
            ui.add_enabled_ui(enabled.unwrap_or(true), |ui| {
                egui::ComboBox::from_label(label)
                    .selected_text(current.clone())
                    .show_ui(ui, |ui| {
                        for option in options {
                            let response =
                                ui.selectable_value(current, option.clone(), option.clone());
                            if response.clicked() {
                                changed = true;
                            }
                        }
                    });
            });
            if changed {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(current.clone()),
                );
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::ChangeCommit,
                    Some(current.clone()),
                );
            }
        }
        UiNodeProps::MenuButton { label, enabled } => {
            let mut was_clicked = false;
            let response = ui.add_enabled_ui(enabled.unwrap_or(true), |ui| {
                ui.menu_button(label.clone(), |ui| {
                    if ui.button("Action").clicked() {
                        was_clicked = true;
                        ui.close_menu();
                    }
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
            });
            emit_interaction_events(
                response.response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label.clone()),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if was_clicked {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Click,
                    Some(label),
                );
            }
        }
        UiNodeProps::CollapsingHeader {
            label,
            open,
            enabled,
        } => {
            let key = (document.document_id, entry.node.id);
            let open_state = ui_state
                .node_open_state
                .get(&key)
                .copied()
                .unwrap_or(open.unwrap_or(true));
            let mut header = egui::CollapsingHeader::new(label.clone());
            header = header.default_open(open_state);
            let response = ui.add_enabled_ui(enabled.unwrap_or(true), |ui| {
                header.show(ui, |ui| {
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
                })
            });
            ui_state
                .node_open_state
                .insert(key, response.inner.openness > 0.0);
            emit_interaction_events(
                response.response,
                realm_id,
                document.document_id,
                entry.node.id,
                Some(label),
                node_tooltip,
                node_context_menu,
                ui_events,
            );
        }
        UiNodeProps::ImageButton {
            source,
            size,
            enabled,
        } => {
            if let Some((texture, texture_size)) = resolve_ui_texture(source, ui_state) {
                let size = resolve_size(size, texture_size);
                let image = egui::Image::from_texture(egui::load::SizedTexture::new(texture, size))
                    .fit_to_exact_size(size);
                let response =
                    ui.add_enabled(enabled.unwrap_or(true), egui::ImageButton::new(image));
                emit_interaction_events(
                    response,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    None,
                    node_tooltip,
                    node_context_menu,
                    ui_events,
                );
            } else {
                ui.label("Image missing");
            }
        }
        UiNodeProps::Spinner { size } => {
            let mut spinner = egui::Spinner::new();
            if let Some(size) = size {
                spinner = spinner.size(size.max(4.0));
            }
            ui.add(spinner);
        }
        UiNodeProps::TextEdit {
            value,
            placeholder,
            multiline,
            password,
            char_limit,
            enabled,
        } => {
            let input_key = (document.document_id, entry.node.id);
            let input_id = egui::Id::new(("ui_text_edit", document.document_id, entry.node.id));
            let text = ui_state
                .input_buffers
                .entry(input_key)
                .or_insert_with(|| value.clone());
            if *text != value && !ui.memory(|memory| memory.has_focus(input_id)) {
                *text = value.clone();
            }
            let was_focused = ui.memory(|memory| memory.has_focus(input_id));
            let mut edit = if multiline.unwrap_or(false) {
                egui::TextEdit::multiline(text)
            } else {
                egui::TextEdit::singleline(text)
            }
            .id_source(input_id)
            .password(password.unwrap_or(false));
            if let Some(placeholder) = placeholder {
                edit = edit.hint_text(placeholder);
            }
            if let Some(char_limit) = char_limit {
                edit = edit.char_limit(char_limit);
            }
            let response = ui.add_enabled(enabled.unwrap_or(true), edit);
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                None,
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(text.clone()),
                );
            }
            let is_focused = ui.memory(|memory| memory.has_focus(input_id));
            if !was_focused && is_focused {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Focus,
                    None,
                );
            }
            if was_focused && !is_focused {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Blur,
                    None,
                );
            }
            let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if submitted {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Submit,
                    Some(text.clone()),
                );
            }
            if response.changed() && response.lost_focus() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::ChangeCommit,
                    Some(text.clone()),
                );
            }
        }
        UiNodeProps::Input {
            value,
            placeholder,
            enabled,
        } => {
            let input_key = (document.document_id, entry.node.id);
            let input_id = egui::Id::new(("ui_input", document.document_id, entry.node.id));
            let text = ui_state
                .input_buffers
                .entry(input_key)
                .or_insert_with(|| value.clone());
            if *text != value && !ui.memory(|memory| memory.has_focus(input_id)) {
                *text = value.clone();
            }

            let mut edit = egui::TextEdit::singleline(text).id_source(input_id);
            if let Some(placeholder) = placeholder {
                edit = edit.hint_text(placeholder);
            }
            let enabled = enabled.unwrap_or(true);
            let response = ui.add_enabled(enabled, edit);
            emit_interaction_events(
                response.clone(),
                realm_id,
                document.document_id,
                entry.node.id,
                None,
                node_tooltip,
                node_context_menu,
                ui_events,
            );
            if response.changed() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::Changed,
                    Some(text.clone()),
                );
            }
            if response.changed() && response.lost_focus() {
                push_ui_event(
                    ui_events,
                    realm_id,
                    document.document_id,
                    entry.node.id,
                    UiEventKind::ChangeCommit,
                    Some(text.clone()),
                );
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
                    let size = resolve_size_in_ui(ui, size, target_size);
                    ui_state.target_size_requests.insert(
                        target_id,
                        glam::UVec2::new(
                            size.x.max(1.0).round() as u32,
                            size.y.max(1.0).round() as u32,
                        ),
                    );
                    let texture =
                        egui::load::SizedTexture::new(egui::TextureId::User(target_id), size);
                    ui.add(egui::Image::from_texture(texture).fit_to_exact_size(size));
                } else {
                    ui.label("Target missing");
                }
            }
        },
        UiNodeProps::WidgetRealmViewport { target_id, size } => {
            if let Some(target_size) = ui_state.external_textures.get(&target_id).copied() {
                let size = resolve_size_in_ui(ui, size, target_size);
                ui_state.target_size_requests.insert(
                    target_id,
                    glam::UVec2::new(
                        size.x.max(1.0).round() as u32,
                        size.y.max(1.0).round() as u32,
                    ),
                );
                let texture = egui::load::SizedTexture::new(egui::TextureId::User(target_id), size);
                ui.add(egui::Image::from_texture(texture).fit_to_exact_size(size));
            } else {
                let fallback = resolve_size(size, [320, 180]);
                let (rect, _) = ui.allocate_exact_size(fallback, egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(28, 32, 40));
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

    layout_def = layout_def
        .with_main_align(justify)
        .with_main_wrap(layout.wrap);
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
    if theme_bool(theme, "darkMode").unwrap_or(false) {
        style.visuals = egui::Visuals::dark();
    } else if theme.data.contains_key("darkMode") {
        style.visuals = egui::Visuals::light();
    }

    if let Some(size) = theme_float(theme, "fontSize") {
        for text_style in style.text_styles.values_mut() {
            text_style.size = size;
        }
    }
    if let Some(size) = theme_float(theme, "fontHeading") {
        set_text_style_size(&mut style, egui::TextStyle::Heading, size);
    }
    if let Some(size) = theme_float(theme, "fontBody") {
        set_text_style_size(&mut style, egui::TextStyle::Body, size);
    }
    if let Some(size) = theme_float(theme, "fontMonospace") {
        set_text_style_size(&mut style, egui::TextStyle::Monospace, size);
    }
    if let Some(size) = theme_float(theme, "fontButton") {
        set_text_style_size(&mut style, egui::TextStyle::Button, size);
    }
    if let Some(size) = theme_float(theme, "fontSmall") {
        set_text_style_size(&mut style, egui::TextStyle::Small, size);
    }

    if let Some(color) = theme_color(theme, "textColor") {
        style.visuals.override_text_color = Some(color);
    }
    if let Some(color) = theme_color(theme, "panelFill") {
        style.visuals.panel_fill = color;
    }
    if let Some(color) = theme_color(theme, "windowFill") {
        style.visuals.window_fill = color;
    }
    if let Some(color) = theme_color(theme, "accentColor") {
        style.visuals.selection.bg_fill = color;
    }
    if let Some(color) = theme_color(theme, "selectionStrokeColor") {
        style.visuals.selection.stroke.color = color;
    }
    if let Some(color) = theme_color(theme, "hyperlinkColor") {
        style.visuals.hyperlink_color = color;
    }

    if let Some(value) = theme_float(theme, "spacingItemX") {
        style.spacing.item_spacing.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingItemY") {
        style.spacing.item_spacing.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingButtonX") {
        style.spacing.button_padding.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingButtonY") {
        style.spacing.button_padding.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingWindowX") {
        style.spacing.window_margin.left = value.max(0.0);
        style.spacing.window_margin.right = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingWindowY") {
        style.spacing.window_margin.top = value.max(0.0);
        style.spacing.window_margin.bottom = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingIndent") {
        style.spacing.indent = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingInteractX") {
        style.spacing.interact_size.x = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingInteractY") {
        style.spacing.interact_size.y = value.max(0.0);
    }
    if let Some(value) = theme_float(theme, "spacingSliderWidth") {
        style.spacing.slider_width = value.max(0.0);
    }

    if let Some(value) = theme_float(theme, "roundingWindow") {
        style.visuals.window_rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingMenu") {
        style.visuals.menu_rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetInactive") {
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetHovered") {
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetActive") {
        style.visuals.widgets.active.rounding = egui::Rounding::same(value.max(0.0));
    }
    if let Some(value) = theme_float(theme, "roundingWidgetOpen") {
        style.visuals.widgets.open.rounding = egui::Rounding::same(value.max(0.0));
    }

    if let Some(value) = theme_float(theme, "strokeWindowWidth") {
        style.visuals.window_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWindowColor") {
        style.visuals.window_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetInactiveWidth") {
        style.visuals.widgets.inactive.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetInactiveColor") {
        style.visuals.widgets.inactive.bg_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetHoveredWidth") {
        style.visuals.widgets.hovered.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetHoveredColor") {
        style.visuals.widgets.hovered.bg_stroke.color = color;
    }
    if let Some(value) = theme_float(theme, "strokeWidgetActiveWidth") {
        style.visuals.widgets.active.bg_stroke.width = value.max(0.0);
    }
    if let Some(color) = theme_color(theme, "strokeWidgetActiveColor") {
        style.visuals.widgets.active.bg_stroke.color = color;
    }

    apply_text_style_family_override(&mut style, theme, "fontFamilyProportional", false);
    apply_text_style_family_override(&mut style, theme, "fontFamilyMonospace", true);

    ctx.set_style(style);

    if !theme.font_data.is_empty() || !theme.font_families.is_empty() {
        let mut definitions = egui::FontDefinitions::default();
        for (name, bytes) in &theme.font_data {
            definitions
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(bytes.clone()));
        }
        for (family_key, family_fonts) in &theme.font_families {
            let family = if family_key.eq_ignore_ascii_case("proportional") {
                egui::FontFamily::Proportional
            } else if family_key.eq_ignore_ascii_case("monospace") {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Name(family_key.clone().into())
            };
            definitions.families.insert(family, family_fonts.clone());
        }
        ctx.set_fonts(definitions);
    }
}

fn parse_color_string(value: &str) -> Option<egui::Color32> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        let parsed = match hex.len() {
            6 => u32::from_str_radix(hex, 16).ok().map(|v| (v, 255u8)),
            8 => u32::from_str_radix(hex, 16)
                .ok()
                .map(|v| (v >> 8, (v & 0xFF) as u8)),
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

fn theme_float(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<f32> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::Float(value)) => Some(*value as f32),
        Some(crate::core::ui::types::UiThemeValue::Int(value)) => Some(*value as f32),
        _ => None,
    }
}

fn theme_bool(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<bool> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn theme_color(theme: &crate::core::ui::state::UiThemeState, key: &str) -> Option<egui::Color32> {
    match theme.data.get(key) {
        Some(crate::core::ui::types::UiThemeValue::String(value)) => parse_color_string(value),
        _ => None,
    }
}

fn set_text_style_size(style: &mut egui::Style, text_style: egui::TextStyle, size: f32) {
    if let Some(font_id) = style.text_styles.get_mut(&text_style) {
        font_id.size = size.max(1.0);
    }
}

fn apply_text_style_family_override(
    style: &mut egui::Style,
    theme: &crate::core::ui::state::UiThemeState,
    key: &str,
    monospace_only: bool,
) {
    let Some(crate::core::ui::types::UiThemeValue::String(family_name)) = theme.data.get(key)
    else {
        return;
    };
    let family = if family_name.eq_ignore_ascii_case("proportional") {
        egui::FontFamily::Proportional
    } else if family_name.eq_ignore_ascii_case("monospace") {
        egui::FontFamily::Monospace
    } else {
        egui::FontFamily::Name(family_name.clone().into())
    };
    for (text_style, font_id) in style.text_styles.iter_mut() {
        let is_mono_style = matches!(text_style, egui::TextStyle::Monospace);
        if (monospace_only && is_mono_style) || (!monospace_only && !is_mono_style) {
            font_id.family = family.clone();
        }
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

fn build_custom_frame(
    padding: Option<UiPadding>,
    fill: Option<UiColor>,
    stroke: Option<UiStroke>,
    rounding: Option<f32>,
) -> egui::Frame {
    let mut frame = build_padding_frame(padding);
    if let Some(fill) = fill {
        frame = frame.fill(color_to_color32(fill));
    }
    if let Some(stroke) = stroke {
        frame = frame.stroke(egui::Stroke::new(
            stroke.width.max(0.0),
            color_to_color32(stroke.color),
        ));
    }
    if let Some(rounding) = rounding {
        frame = frame.rounding(egui::Rounding::same(rounding.max(0.0)));
    }
    frame
}

#[allow(clippy::too_many_arguments)]
fn render_split_pane(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
    direction: UiSplitDirection,
    ratio: Option<f32>,
    resizable: bool,
    min_a: f32,
    max_a: Option<f32>,
    min_b: f32,
    max_b: Option<f32>,
) {
    let container_rect = ui.available_rect_before_wrap();
    let key = (document.document_id, entry.node.id);
    let mut current_ratio = *ui_state
        .split_ratios
        .entry(key)
        .or_insert(ratio.unwrap_or(0.5).clamp(0.05, 0.95));

    let total_primary = if direction == UiSplitDirection::Horizontal {
        container_rect.width()
    } else {
        container_rect.height()
    }
    .max(1.0);
    let mut primary_a = total_primary * current_ratio;
    let mut min_primary_a = min_a.max(0.0);
    if let Some(max_a) = max_a {
        min_primary_a = min_primary_a.min(max_a.max(0.0));
    }
    let mut max_primary_a = total_primary - min_b.max(0.0);
    if let Some(max_b) = max_b {
        max_primary_a = max_primary_a.min(total_primary - max_b.max(0.0));
    }
    if let Some(max_a) = max_a {
        max_primary_a = max_primary_a.min(max_a.max(0.0));
    }
    if max_primary_a < min_primary_a {
        max_primary_a = min_primary_a;
    }
    primary_a = primary_a.clamp(min_primary_a, max_primary_a);
    current_ratio = (primary_a / total_primary).clamp(0.01, 0.99);

    let handle_size = 6.0;
    let (rect_a, handle_rect, rect_b) = if direction == UiSplitDirection::Horizontal {
        let rect_a = egui::Rect::from_min_max(
            container_rect.min,
            egui::pos2(container_rect.min.x + primary_a, container_rect.max.y),
        );
        let handle_rect = egui::Rect::from_min_max(
            egui::pos2(rect_a.max.x, container_rect.min.y),
            egui::pos2(rect_a.max.x + handle_size, container_rect.max.y),
        );
        let rect_b = egui::Rect::from_min_max(
            egui::pos2(handle_rect.max.x, container_rect.min.y),
            container_rect.max,
        );
        (rect_a, handle_rect, rect_b)
    } else {
        let rect_a = egui::Rect::from_min_max(
            container_rect.min,
            egui::pos2(container_rect.max.x, container_rect.min.y + primary_a),
        );
        let handle_rect = egui::Rect::from_min_max(
            egui::pos2(container_rect.min.x, rect_a.max.y),
            egui::pos2(container_rect.max.x, rect_a.max.y + handle_size),
        );
        let rect_b = egui::Rect::from_min_max(
            egui::pos2(container_rect.min.x, handle_rect.max.y),
            container_rect.max,
        );
        (rect_a, handle_rect, rect_b)
    };

    let sense = if resizable {
        egui::Sense::click_and_drag()
    } else {
        egui::Sense::hover()
    };
    let handle_id = egui::Id::new((document.document_id, entry.node.id, "split-handle"));
    let response = ui.interact(handle_rect, handle_id, sense);
    if resizable && (response.hovered() || response.dragged()) {
        ui.output_mut(|output| {
            output.cursor_icon = if direction == UiSplitDirection::Horizontal {
                egui::CursorIcon::ResizeHorizontal
            } else {
                egui::CursorIcon::ResizeVertical
            };
        });
    }
    if response.dragged() && resizable {
        let delta = if direction == UiSplitDirection::Horizontal {
            response.drag_delta().x
        } else {
            response.drag_delta().y
        };
        let moved = (primary_a + delta).clamp(min_primary_a, max_primary_a);
        current_ratio = (moved / total_primary).clamp(0.01, 0.99);
    }
    ui.painter()
        .rect_filled(handle_rect, 0.0, egui::Color32::from_gray(110));
    ui_state.split_ratios.insert(key, current_ratio);

    let children = child_ids_ordered(document, entry.node.id, &entry.children);
    if let Some(first_id) = children.first()
        && let Some(child) = document.nodes.get(first_id)
    {
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect_a), |ui| {
            render_node(
                ui,
                document,
                child,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
            );
        });
    }
    if let Some(second_id) = children.get(1)
        && let Some(child) = document.nodes.get(second_id)
    {
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect_b), |ui| {
            render_node(
                ui,
                document,
                child,
                ui_state,
                realm_id,
                ui_events,
                time_seconds,
            );
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_scene_node(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
    size: Option<UiSize>,
    zoom_min: f32,
    zoom_max: f32,
    pan_enabled: bool,
) {
    let scene_key = (document.document_id, entry.node.id);
    let initial_scene = ui_state.scene_state.get(&scene_key).copied().unwrap_or(
        crate::core::ui::state::UiSceneState {
            pan: glam::Vec2::ZERO,
            zoom: 1.0,
        },
    );
    let mut zoom = initial_scene.zoom;
    let mut pan = initial_scene.pan;
    let fallback = resolve_size(size, [320, 220]);
    let (rect, response) = ui.allocate_exact_size(
        fallback,
        if pan_enabled {
            egui::Sense::drag()
        } else {
            egui::Sense::hover()
        },
    );
    let zoom_delta = ui.input(|i| i.zoom_delta());
    if (zoom_delta - 1.0).abs() > f32::EPSILON {
        zoom = (zoom * zoom_delta).clamp(zoom_min.max(0.01), zoom_max.max(zoom_min));
    }
    if response.dragged() && pan_enabled {
        let delta = response.drag_delta();
        pan += glam::Vec2::new(delta.x, delta.y);
    }
    let content_rect = rect.translate(egui::vec2(pan.x, pan.y));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        ui.set_clip_rect(rect);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing *= zoom.max(0.01);
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
    });
    ui_state.scene_state.insert(
        scene_key,
        crate::core::ui::state::UiSceneState { pan, zoom },
    );
}

fn child_ids_ordered(
    document: &UiDocument,
    parent_id: UiNodeId,
    fallback: &[UiNodeId],
) -> Vec<UiNodeId> {
    document
        .ordered_children
        .get(&parent_id)
        .cloned()
        .unwrap_or_else(|| fallback.to_vec())
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

fn resolve_size_in_ui(ui: &egui::Ui, size: Option<UiSize>, fallback: [u32; 2]) -> egui::Vec2 {
    let fallback = egui::vec2(fallback[0] as f32, fallback[1] as f32);
    let available = ui.available_size();
    let Some(size) = size else { return fallback };

    let width = match size.width {
        UiLength::Auto => fallback.x,
        UiLength::Fill => available.x.max(0.0),
        UiLength::Px(value) => value,
    };
    let height = match size.height {
        UiLength::Auto => fallback.y,
        UiLength::Fill => available.y.max(0.0),
        UiLength::Px(value) => value,
    };
    egui::vec2(width.max(0.0), height.max(0.0))
}

fn resolve_ui_texture(
    source: UiImageSource,
    ui_state: &UiState,
) -> Option<(egui::TextureId, [u32; 2])> {
    match source {
        UiImageSource::UiImage(image_id) => {
            let record = ui_state.images.get(&image_id)?;
            let texture = record.texture.as_ref()?;
            Some((texture.id(), record.size))
        }
        UiImageSource::Target(target_id) => {
            let size = ui_state.external_textures.get(&target_id).copied()?;
            Some((egui::TextureId::User(target_id), size))
        }
    }
}

fn emit_interaction_events(
    response: egui::Response,
    realm_id: RealmId,
    document_id: u32,
    node_id: u32,
    label: Option<String>,
    tooltip: Option<&str>,
    context_menu: Option<&[String]>,
    ui_events: &mut Vec<UiEvent>,
) {
    if let Some(tooltip) = tooltip {
        response.clone().on_hover_text(tooltip.to_string());
    }
    if let Some(items) = context_menu {
        let context_items: Vec<String> = items.to_vec();
        let _ = response.clone().context_menu(|ui| {
            for item in context_items {
                if ui.button(item.clone()).clicked() {
                    push_ui_event(
                        ui_events,
                        realm_id,
                        document_id,
                        node_id,
                        UiEventKind::Click,
                        Some(format!("context:{}", item)),
                    );
                    ui.close_menu();
                }
            }
        });
    }

    if response.clicked() {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::Click,
            label.clone(),
        );
    }
    if response.double_clicked() {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::DoubleClick,
            label.clone(),
        );
    }
    if response.is_pointer_button_down_on() {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::Pressed,
            label.clone(),
        );
    }
    if response.clicked_elsewhere() {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::Released,
            label.clone(),
        );
    }
    if response.hovered() {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::HoverEnter,
            label.clone(),
        );
    } else {
        push_ui_event(
            ui_events,
            realm_id,
            document_id,
            node_id,
            UiEventKind::HoverLeave,
            label,
        );
    }
}

fn push_ui_event(
    ui_events: &mut Vec<UiEvent>,
    realm_id: RealmId,
    document_id: u32,
    node_id: u32,
    kind: UiEventKind,
    label: Option<String>,
) {
    ui_events.push(UiEvent {
        realm_id: realm_id.0,
        document_id,
        node_id,
        kind,
        label,
    });
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
