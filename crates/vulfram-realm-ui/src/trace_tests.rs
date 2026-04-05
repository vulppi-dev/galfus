use super::*;
use crate::{UiNode, UiNodeKind, UiNodeProps};

#[test]
fn resolve_traced_pointer_dispatch_prefers_source_realm_uv_size() {
    let documents = ui_documents();
    let dispatch = resolve_traced_pointer_dispatch(
        &documents,
        &UiTracedPointerContext {
            trace_realm_id: RealmId(1),
            trace_source_realm_id: Some(RealmId(2)),
            uv: Some(glam::vec2(0.5, 0.25)),
            cursor_position: None,
            realm_output_size: Some(glam::uvec2(200, 100)),
            connector_source_size: Some(glam::uvec2(400, 200)),
        },
    )
    .expect("dispatch should resolve");

    assert_eq!(dispatch.realm_id, RealmId(2));
    assert_eq!(dispatch.document_id, 20);
    assert_eq!(dispatch.pos, glam::vec2(100.0, 25.0));
    assert_eq!(dispatch.realm_size, glam::uvec2(200, 100));
}

#[test]
fn resolve_traced_pointer_dispatch_falls_back_to_cursor_position() {
    let documents = ui_documents();
    let dispatch = resolve_traced_pointer_dispatch(
        &documents,
        &UiTracedPointerContext {
            trace_realm_id: RealmId(1),
            trace_source_realm_id: None,
            uv: None,
            cursor_position: Some(glam::vec2(20.0, 30.0)),
            realm_output_size: Some(glam::uvec2(100, 100)),
            connector_source_size: None,
        },
    )
    .expect("dispatch should resolve");

    assert_eq!(dispatch.realm_id, RealmId(1));
    assert_eq!(dispatch.document_id, 10);
    assert_eq!(dispatch.pos, glam::vec2(20.0, 30.0));
}

fn ui_documents() -> HashMap<UiDocumentId, UiDocument> {
    let mut root = UiDocument::new(10, RealmId(1), glam::vec4(0.0, 0.0, 100.0, 100.0));
    root.add_node(None, text_node(1, Some(0)), None)
        .expect("root node");

    let mut source = UiDocument::new(20, RealmId(2), glam::vec4(0.0, 0.0, 200.0, 100.0));
    source
        .add_node(None, text_node(2, Some(1)), None)
        .expect("source node");

    HashMap::from([(10, root), (20, source)])
}

fn text_node(node_id: u32, z_index: Option<i32>) -> UiNode {
    UiNode {
        id: node_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "text".into(),
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
