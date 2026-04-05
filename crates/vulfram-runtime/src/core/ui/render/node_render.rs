use super::node_helpers::*;
use super::node_render_controls::render_node_inner_controls;
use super::node_render_layout::render_node_inner_layout;
use super::node_render_media::render_node_inner_media;
use super::*;

pub(super) fn render_children(
    ui: &mut egui::Ui,
    document: &UiDocument,
    parent: Option<UiNodeId>,
    children: &[UiNodeId],
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
) {
    let ordered: &[UiNodeId] = match parent {
        Some(parent_id) => document
            .ordered_children
            .get(&parent_id)
            .map(Vec::as_slice)
            .unwrap_or(children),
        None => {
            if document.ordered_root.is_empty() {
                children
            } else {
                document.ordered_root.as_slice()
            }
        }
    };

    for node_id in ordered.iter().copied() {
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

pub(super) fn render_node(
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
    let mut translate_y: f32 = 0.0;
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
    let props = entry.node.props.clone();

    if render_node_inner_layout(
        ui,
        document,
        entry,
        ui_state,
        realm_id,
        ui_events,
        time_seconds,
        node_tooltip,
        node_context_menu,
        props.clone(),
    ) {
        return;
    }
    if render_node_inner_controls(
        ui,
        document,
        entry,
        ui_state,
        realm_id,
        ui_events,
        time_seconds,
        node_tooltip,
        node_context_menu,
        props.clone(),
    ) {
        return;
    }
    let _ = render_node_inner_media(
        ui,
        document,
        entry,
        ui_state,
        realm_id,
        ui_events,
        time_seconds,
        node_tooltip,
        node_context_menu,
        props,
    );
}
