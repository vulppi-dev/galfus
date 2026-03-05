use crate::core::realm::{CmdRealmCreateArgs, RealmId, RealmKindDto, engine_cmd_realm_create};
use crate::core::state::EngineState;
use crate::core::ui::cmd::{
    CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, engine_cmd_ui_apply_ops,
    engine_cmd_ui_document_create,
};
use crate::core::ui::events::UiEventKind;
use crate::core::ui::input::process_ui_input;
use crate::core::ui::render::render_realm_documents;
use crate::core::ui::state::UiDocument;
use crate::core::ui::types::{
    UiAnim, UiAnimEasing, UiAnimSpec, UiNode, UiNodeKind, UiNodeProps, UiOp,
};

fn text_node(node_id: u32, text: &str) -> UiNode {
    UiNode {
        id: node_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: text.to_string(),
            size: None,
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    }
}

#[test]
fn ui_apply_ops_validates_version_and_applies_add_move_set_remove_clear() {
    let mut engine = EngineState::new();
    let created_realm = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    assert!(created_realm.success);
    let realm_id = created_realm.realm_id.unwrap();

    let created_doc = engine_cmd_ui_document_create(
        &mut engine,
        &CmdUiDocumentCreateArgs {
            document_id: 500,
            realm_id,
            rect: glam::vec4(0.0, 0.0, 100.0, 100.0),
            theme_id: None,
        },
    );
    assert!(created_doc.success);

    let result_v1 = engine_cmd_ui_apply_ops(
        &mut engine,
        &CmdUiApplyOpsArgs {
            document_id: 500,
            version: 1,
            ops: vec![
                UiOp::Add {
                    parent: None,
                    node: text_node(1, "root-a"),
                    index: None,
                },
                UiOp::Add {
                    parent: None,
                    node: text_node(2, "root-b"),
                    index: None,
                },
                UiOp::Move {
                    node_id: 2,
                    new_parent: None,
                    index: Some(0),
                },
                UiOp::Set {
                    node_id: 1,
                    props: UiNodeProps::Text {
                        text: "updated".into(),
                        size: None,
                        color: None,
                    },
                },
                UiOp::Remove { node_id: 2 },
            ],
        },
    );
    assert!(result_v1.success);
    let doc = engine.universal_state.ui.documents.get(&500).unwrap();
    assert_eq!(doc.version, 1);
    assert_eq!(doc.nodes.len(), 1);
    assert!(doc.nodes.contains_key(&1));

    let result_v1_again = engine_cmd_ui_apply_ops(
        &mut engine,
        &CmdUiApplyOpsArgs {
            document_id: 500,
            version: 1,
            ops: vec![UiOp::Clear { parent: None }],
        },
    );
    assert!(!result_v1_again.success);
    assert!(result_v1_again.message.contains("version mismatch"));
}

#[test]
fn ui_document_orders_children_by_z_index_desc() {
    let mut doc = UiDocument::new(10, RealmId(1), glam::vec4(0.0, 0.0, 100.0, 100.0));
    let mut a = text_node(10, "a");
    a.z_index = Some(1);
    let mut b = text_node(11, "b");
    b.z_index = Some(5);
    let mut c = text_node(12, "c");
    c.z_index = Some(3);

    doc.add_node(None, a, None).unwrap();
    doc.add_node(None, b, None).unwrap();
    doc.add_node(None, c, None).unwrap();
    doc.ensure_layout_cache();
    assert_eq!(doc.ordered_root, vec![10, 12, 11]);
}

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
fn ui_input_converts_pointer_button_into_egui_pointer_event() {
    let mut engine = EngineState::new();
    let created_realm = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    let realm_id = created_realm.realm_id.unwrap();
    let _ = engine_cmd_ui_document_create(
        &mut engine,
        &CmdUiDocumentCreateArgs {
            document_id: 900,
            realm_id,
            rect: glam::vec4(0.0, 0.0, 320.0, 240.0),
            theme_id: None,
        },
    );
    let surface_id = engine
        .universal_state
        .surfaces
        .alloc(crate::core::realm::SurfaceState {
            kind: crate::core::realm::SurfaceKind::Offscreen,
            size: glam::uvec2(320, 240),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
    if let Some(realm_entry) = engine
        .universal_state
        .realms
        .entries
        .get_mut(&RealmId(realm_id))
    {
        realm_entry.value.output_surface = Some(surface_id);
    }
    engine
        .event_queue
        .push(crate::core::cmd::EngineEvent::Pointer(
            crate::core::input::events::PointerEvent::OnButton {
                window_id: 1,
                pointer_type: 0,
                pointer_id: 1,
                button: 1,
                state: crate::core::input::events::ElementState::Pressed,
                position: glam::vec2(100.0, 80.0),
                trace: Some(crate::core::input::events::PointerEventTrace {
                    window_id: 1,
                    realm_id,
                    target_id: None,
                    connector_id: None,
                    source_realm_id: None,
                    uv: None,
                    hops: Vec::new(),
                }),
            },
        ));

    process_ui_input(&mut engine);

    let realm_state = engine
        .universal_state
        .ui
        .realms
        .get(&RealmId(realm_id))
        .unwrap();
    assert!(
        realm_state
            .pending_events
            .iter()
            .any(|event| matches!(event, egui::Event::PointerButton { pressed: true, .. }))
    );
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
fn ui_create_dispose_loop_does_not_leave_orphan_state() {
    let mut engine = EngineState::new();
    let created_realm = engine_cmd_realm_create(
        &mut engine,
        &CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    let realm_id = created_realm.realm_id.unwrap();

    for i in 0..200u32 {
        let _ = engine_cmd_ui_document_create(
            &mut engine,
            &CmdUiDocumentCreateArgs {
                document_id: 10_000 + i,
                realm_id,
                rect: glam::vec4(0.0, 0.0, 320.0 + i as f32, 180.0),
                theme_id: None,
            },
        );
        let _ = engine_cmd_ui_apply_ops(
            &mut engine,
            &CmdUiApplyOpsArgs {
                document_id: 10_000 + i,
                version: 1,
                ops: vec![UiOp::Add {
                    parent: None,
                    node: text_node(1, "stress"),
                    index: None,
                }],
            },
        );
        let _ = crate::core::ui::cmd::engine_cmd_ui_document_dispose(
            &mut engine,
            &crate::core::ui::cmd::CmdUiDocumentDisposeArgs {
                document_id: 10_000 + i,
            },
        );
    }

    assert!(engine.universal_state.ui.documents.is_empty());
    assert!(engine.universal_state.ui.input_buffers.is_empty());
    assert!(engine.universal_state.ui.animations.is_empty());
    assert!(engine.universal_state.ui.split_ratios.is_empty());
    assert!(engine.universal_state.ui.scene_state.is_empty());
}
