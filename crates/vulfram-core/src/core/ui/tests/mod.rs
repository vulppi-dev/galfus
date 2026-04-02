use crate::core::ui::types::{UiNode, UiNodeKind, UiNodeProps};

mod input;
mod lifecycle;
mod rendering;

pub(super) fn text_node(node_id: u32, text: &str) -> UiNode {
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
