use super::node_helpers::*;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_node_inner_layout(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
    _node_tooltip: Option<&str>,
    _node_context_menu: Option<&[String]>,
    props: UiNodeProps,
) -> bool {
    match props {
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
        _ => return false,
    }
    true
}
