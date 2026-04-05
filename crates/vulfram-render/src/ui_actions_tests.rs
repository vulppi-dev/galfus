use super::{UiPlatformAction, collect_platform_actions};

#[test]
fn collects_root_viewport_window_actions() {
    let viewport = egui::ViewportOutput {
        parent: egui::ViewportId::ROOT,
        class: egui::ViewportClass::Root,
        builder: egui::ViewportBuilder::default().with_title("Main"),
        viewport_ui_cb: None,
        commands: vec![egui::ViewportCommand::Title("Renamed".into())],
        repaint_delay: std::time::Duration::ZERO,
    };
    let mut output = egui::FullOutput::default();
    output
        .viewport_output
        .insert(egui::ViewportId::ROOT, viewport);

    let actions = collect_platform_actions(&output, 7, 11);
    assert!(actions.iter().any(|action| matches!(
        action,
        UiPlatformAction::SetWindowTitle { window_id, title }
        if *window_id == 7 && title == "Renamed"
    )));
}

#[test]
fn emits_embedded_fallback_for_non_root_viewport() {
    let child = egui::ViewportOutput {
        parent: egui::ViewportId::ROOT,
        class: egui::ViewportClass::Deferred,
        builder: egui::ViewportBuilder::default(),
        viewport_ui_cb: None,
        commands: vec![egui::ViewportCommand::Close],
        repaint_delay: std::time::Duration::ZERO,
    };
    let mut output = egui::FullOutput::default();
    output
        .viewport_output
        .insert(egui::ViewportId::from_hash_of("child"), child);

    let actions = collect_platform_actions(&output, 3, 9);
    assert!(actions.iter().any(|action| matches!(
        action,
        UiPlatformAction::EmitViewportFallbackEmbedded { window_id, realm_id, .. }
        if *window_id == 3 && *realm_id == 9
    )));
    assert!(actions.iter().any(|action| matches!(
        action,
        UiPlatformAction::EmitViewportCommand { window_id, realm_id, .. }
        if *window_id == 3 && *realm_id == 9
    )));
}
