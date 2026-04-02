use super::*;
use crate::{UiNode, UiNodeProps};
use vulfram_types::RealmId;

#[test]
fn build_tree_node_uses_ordered_children_when_available() {
    let mut doc = UiDocument::new(1, RealmId(7), glam::vec4(0.0, 0.0, 100.0, 100.0));
    doc.add_node(None, text_node(10, "root", Some(0)), None)
        .expect("root should be added");
    doc.add_node(Some(10), text_node(11, "back", Some(-1)), None)
        .expect("child should be added");
    doc.add_node(Some(10), text_node(12, "front", Some(1)), None)
        .expect("child should be added");
    doc.ensure_layout_cache();

    let tree = build_tree_node(&doc, 10).expect("tree should exist");
    assert_eq!(
        tree.children
            .iter()
            .map(|child| child.node_id)
            .collect::<Vec<_>>(),
        vec![11, 12]
    );
}

fn text_node(node_id: UiNodeId, text: &str, z_index: Option<i32>) -> UiNode {
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
        z_index,
    }
}
