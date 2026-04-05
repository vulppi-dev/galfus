use crate::core::realm::{CmdRealmCreateArgs, RealmId, RealmKindDto, engine_cmd_realm_create};
use crate::core::state::EngineState;
use crate::core::ui::cmd::{
    CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, engine_cmd_ui_apply_ops,
    engine_cmd_ui_document_create,
};
use crate::core::ui::state::UiDocument;
use crate::core::ui::types::UiOp;

use super::text_node;

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
    let realm_id = created_realm.realm_id.expect("realm id");

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
                    props: crate::core::ui::types::UiNodeProps::Text {
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
    let doc = engine
        .universal_state
        .interaction
        .ui
        .documents
        .get(&500)
        .expect("document");
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

    doc.add_node(None, a, None).expect("add a");
    doc.add_node(None, b, None).expect("add b");
    doc.add_node(None, c, None).expect("add c");
    doc.ensure_layout_cache();
    assert_eq!(doc.ordered_root, vec![10, 12, 11]);
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
    let realm_id = created_realm.realm_id.expect("realm id");

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

    assert!(engine.universal_state.interaction.ui.documents.is_empty());
    assert!(
        engine
            .universal_state
            .interaction
            .ui
            .input_buffers
            .is_empty()
    );
    assert!(engine.universal_state.interaction.ui.animations.is_empty());
    assert!(
        engine
            .universal_state
            .interaction
            .ui
            .split_ratios
            .is_empty()
    );
    assert!(engine.universal_state.interaction.ui.scene_state.is_empty());
}
