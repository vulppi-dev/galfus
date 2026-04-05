use crate::core::realm::RealmId;
use crate::core::ui::events::UiEventKind;
use crate::core::ui::render::render_realm_documents;
use crate::core::ui::state::UiDocument;
use crate::core::ui::types::{UiAnim, UiAnimEasing, UiAnimSpec, UiNode, UiNodeKind, UiNodeProps};

use super::text_node;

#[test]
fn ui_animation_emits_anim_complete_once() {
    let mut ui_state = crate::core::ui::UiState::default();
    let realm_id = RealmId(77);
    ui_state.ensure_realm(realm_id);
    let mut doc = UiDocument::new(300, realm_id, glam::vec4(0.0, 0.0, 400.0, 300.0));
    let mut animated = text_node(1000, "anim");
    animated.anim = Some(UiAnim {
        opacity: Some(UiAnimSpec {
            from: 0.0,
            to: 1.0,
            duration_ms: 100,
            easing: UiAnimEasing::Linear,
        }),
        translate_y: None,
    });
    let _ = doc.add_node(None, animated, None);
    ui_state.documents.insert(300, doc);

    let ctx = egui::Context::default();
    let mut ui_events = Vec::new();

    let _ = ctx.run(Default::default(), |ctx| {
        render_realm_documents(
            ctx,
            &mut ui_state,
            realm_id,
            glam::uvec2(400, 300),
            &mut ui_events,
            0.0,
        );
    });
    assert!(ui_events.is_empty());

    let _ = ctx.run(Default::default(), |ctx| {
        render_realm_documents(
            ctx,
            &mut ui_state,
            realm_id,
            glam::uvec2(400, 300),
            &mut ui_events,
            0.2,
        );
    });
    assert_eq!(
        ui_events
            .iter()
            .filter(|event| event.kind == UiEventKind::AnimComplete)
            .count(),
        1
    );
}

#[test]
fn ui_shape_hash_generates_render_fingerprint() {
    let mut ui_state = crate::core::ui::UiState::default();
    let realm_id = RealmId(1234);
    ui_state.ensure_realm(realm_id);
    let mut doc = UiDocument::new(77, realm_id, glam::vec4(0.0, 0.0, 300.0, 120.0));
    let _ = doc.add_node(None, text_node(1, "golden"), None);
    ui_state.documents.insert(77, doc);

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        let mut ui_events = Vec::new();
        render_realm_documents(
            ctx,
            &mut ui_state,
            realm_id,
            glam::uvec2(300, 120),
            &mut ui_events,
            0.0,
        );
    });
    let output_a = ctx.run(Default::default(), |ctx| {
        let mut ui_events = Vec::new();
        render_realm_documents(
            ctx,
            &mut ui_state,
            realm_id,
            glam::uvec2(300, 120),
            &mut ui_events,
            0.0,
        );
    });
    let hash_a = crate::core::ui::render::hash_shapes(&output_a.shapes);

    let output_b = ctx.run(Default::default(), |ctx| {
        let mut ui_events = Vec::new();
        render_realm_documents(
            ctx,
            &mut ui_state,
            realm_id,
            glam::uvec2(300, 120),
            &mut ui_events,
            0.0,
        );
    });
    let hash_b = crate::core::ui::render::hash_shapes(&output_b.shapes);
    assert_ne!(hash_a, 0);
    assert_ne!(hash_b, 0);
}

#[test]
fn ui_split_pane_resizable_reacts_to_pointer_drag() {
    let mut ui_state = crate::core::ui::UiState::default();
    let realm_id = RealmId(9001);
    ui_state.ensure_realm(realm_id);
    let mut doc = UiDocument::new(901, realm_id, glam::vec4(0.0, 0.0, 400.0, 200.0));
    let split = UiNode {
        id: 1,
        kind: UiNodeKind::SplitPane,
        props: UiNodeProps::SplitPane {
            direction: crate::core::ui::types::UiSplitDirection::Horizontal,
            ratio: Some(0.5),
            resizable: Some(true),
            min_a: Some(10.0),
            max_a: None,
            min_b: Some(10.0),
            max_b: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };
    let _ = doc.add_node(None, split, None);
    let _ = doc.add_node(Some(1), text_node(2, "left"), None);
    let _ = doc.add_node(Some(1), text_node(3, "right"), None);
    ui_state.documents.insert(901, doc);

    let ctx = egui::Context::default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 200.0),
            )),
            events: vec![
                egui::Event::PointerMoved(egui::pos2(202.0, 40.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(202.0, 40.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            ..Default::default()
        },
        |ctx| {
            let mut ui_events = Vec::new();
            render_realm_documents(
                ctx,
                &mut ui_state,
                realm_id,
                glam::uvec2(400, 200),
                &mut ui_events,
                0.0,
            );
        },
    );
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 200.0),
            )),
            events: vec![
                egui::Event::PointerMoved(egui::pos2(260.0, 40.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(260.0, 40.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            ..Default::default()
        },
        |ctx| {
            let mut ui_events = Vec::new();
            render_realm_documents(
                ctx,
                &mut ui_state,
                realm_id,
                glam::uvec2(400, 200),
                &mut ui_events,
                0.016,
            );
        },
    );

    let ratio = ui_state
        .split_ratios
        .get(&(901, 1))
        .copied()
        .unwrap_or(-1.0);
    assert!((0.0..=1.0).contains(&ratio));
}
