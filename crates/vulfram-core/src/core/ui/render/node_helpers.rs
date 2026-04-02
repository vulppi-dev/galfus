use super::node_render::render_node;
use super::*;
pub(super) fn render_layout(
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

pub(super) fn resolve_anim(
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

pub(super) fn apply_easing(value: f32, easing: UiAnimEasing) -> f32 {
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

pub(super) fn apply_size(ui: &mut egui::Ui, size: Option<UiSize>) {
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

pub(super) fn build_padding_frame(padding: Option<UiPadding>) -> egui::Frame {
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

pub(super) fn build_custom_frame(
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
pub(super) fn render_split_pane(
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
pub(super) fn render_scene_node(
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

pub(super) fn child_ids_ordered<'a>(
    document: &'a UiDocument,
    parent_id: UiNodeId,
    fallback: &'a [UiNodeId],
) -> &'a [UiNodeId] {
    document
        .ordered_children
        .get(&parent_id)
        .map(Vec::as_slice)
        .unwrap_or(fallback)
}

pub(super) fn resolve_size(size: Option<UiSize>, fallback: [u32; 2]) -> egui::Vec2 {
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

pub(super) fn resolve_size_in_ui(
    ui: &egui::Ui,
    size: Option<UiSize>,
    fallback: [u32; 2],
) -> egui::Vec2 {
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

pub(super) fn resolve_ui_texture(
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

pub(super) fn emit_interaction_events(
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
        let _ = response.clone().context_menu(|ui| {
            for item in items {
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

pub(super) fn push_ui_event(
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

pub(super) fn color_to_color32(color: UiColor) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}
