use super::node_helpers::*;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_node_inner_controls(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    time_seconds: f64,
    node_tooltip: Option<&str>,
    node_context_menu: Option<&[String]>,
    props: UiNodeProps,
) -> bool {
    match props {
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
        _ => return false,
    }
    true
}
