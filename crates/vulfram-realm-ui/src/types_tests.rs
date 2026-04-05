use super::*;

#[test]
fn ui_layout_defaults_to_column_start() {
    let layout = UiLayout::default();
    assert_eq!(layout.direction, UiLayoutDirection::Column);
    assert_eq!(layout.align, UiAlign::Start);
    assert_eq!(layout.justify, UiAlign::Start);
}

#[test]
fn ui_eventful_node_round_trips_through_json() {
    let node = UiNode {
        id: 10,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "hello".into(),
            size: Some(14.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: Some(true),
        visible: Some(true),
        opacity: Some(1.0),
        z_index: Some(2),
    };

    let json = serde_json::to_string(&node).expect("node should encode");
    let decoded: UiNode = serde_json::from_str(&json).expect("node should decode");
    assert_eq!(decoded, node);
}
