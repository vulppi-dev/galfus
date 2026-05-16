use super::node_helpers::*;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn render_node_inner_media(
    ui: &mut egui::Ui,
    document: &UiDocument,
    entry: &UiNodeEntry,
    ui_state: &mut UiState,
    realm_id: RealmId,
    ui_events: &mut Vec<UiEvent>,
    _time_seconds: f64,
    node_tooltip: Option<&str>,
    node_context_menu: Option<&[String]>,
    props: UiNodeProps,
) -> bool {
    match props {
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
        _ => return false,
    }
    true
}
