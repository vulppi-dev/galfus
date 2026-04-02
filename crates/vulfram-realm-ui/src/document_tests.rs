use super::*;
use crate::{UiNodeKind, UiNodeProps, UiOp};

fn text_node(node_id: UiNodeId, text: &str) -> UiNode {
    UiNode {
        id: node_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: text.into(),
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
fn ui_document_add_move_and_remove_nodes() {
    let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));

    doc.add_node(None, text_node(10, "root"), None)
        .expect("root should be added");
    doc.add_node(Some(10), text_node(11, "child"), None)
        .expect("child should be added");
    doc.move_node(11, None, Some(0)).expect("move should work");

    assert_eq!(doc.root_children, vec![11, 10]);

    doc.remove_node(10).expect("remove should work");
    assert!(!doc.nodes.contains_key(&10));
}

#[test]
fn ui_document_apply_ops_rolls_back_on_error() {
    let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
    doc.add_node(None, text_node(10, "root"), None)
        .expect("root should be added");

    let result = doc.apply_ops(
        2,
        &[
            UiOp::Add {
                parent: Some(10),
                node: text_node(11, "child"),
                index: None,
            },
            UiOp::Set {
                node_id: 999,
                props: UiNodeProps::Text {
                    text: "broken".into(),
                    size: None,
                    color: None,
                },
            },
        ],
    );

    assert!(result.is_err());
    assert!(!doc.nodes.contains_key(&11));
    assert_eq!(doc.version, 0);
    assert_eq!(
        doc.nodes.get(&10).map(|entry| entry.children.len()),
        Some(0)
    );
}

#[test]
fn ui_document_apply_ops_reports_removed_nodes() {
    let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
    doc.add_node(None, text_node(10, "root"), None)
        .expect("root should be added");
    doc.add_node(Some(10), text_node(11, "child"), None)
        .expect("child should be added");

    let result = doc
        .apply_ops(2, &[UiOp::Remove { node_id: 10 }])
        .expect("remove should work");

    assert_eq!(result.version, Some(2));
    assert_eq!(result.removed_nodes, vec![10, 11]);
    assert!(doc.nodes.is_empty());
}

#[test]
fn ui_document_layout_cache_sorts_by_z_index() {
    let mut doc = UiDocument::new(1, RealmId(2), glam::vec4(0.0, 0.0, 100.0, 100.0));
    let mut a = text_node(10, "a");
    a.z_index = Some(10);
    let mut b = text_node(11, "b");
    b.z_index = Some(-1);

    doc.add_node(None, a, None).expect("a should be added");
    doc.add_node(None, b, None).expect("b should be added");
    doc.ensure_layout_cache();

    assert_eq!(doc.ordered_root, vec![11, 10]);
}
