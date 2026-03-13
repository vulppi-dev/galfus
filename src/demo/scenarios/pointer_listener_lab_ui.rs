use glam::Vec2;

use crate::core::ui::types::{
    UiColor, UiLayout, UiLayoutDirection, UiLength, UiNode, UiNodeKind, UiNodeProps, UiOp,
    UiPadding, UiSize, UiSplitDirection,
};

pub(super) fn build_ui_tree(
    root_split_id: u32,
    top_panel_id: u32,
    bottom_panel_id: u32,
    title_text_id: u32,
    main_text_id: u32,
    inner_text_id: u32,
    viewport_id: u32,
    inner_target_id: u64,
) -> Vec<UiOp> {
    vec![
        UiOp::Add {
            parent: None,
            node: UiNode {
                id: root_split_id,
                kind: UiNodeKind::SplitPane,
                props: UiNodeProps::SplitPane {
                    direction: UiSplitDirection::Vertical,
                    ratio: Some(0.45),
                    resizable: Some(false),
                    min_a: Some(120.0),
                    max_a: None,
                    min_b: Some(120.0),
                    max_b: None,
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(200),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(root_split_id),
            node: UiNode {
                id: top_panel_id,
                kind: UiNodeKind::Frame,
                props: UiNodeProps::Frame {
                    padding: Some(UiPadding {
                        left: 14.0,
                        top: 10.0,
                        right: 14.0,
                        bottom: 10.0,
                    }),
                    fill: Some(UiColor {
                        r: 244,
                        g: 246,
                        b: 250,
                        a: 255,
                    }),
                    stroke: None,
                    rounding: Some(6.0),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(201),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(top_panel_id),
            node: UiNode {
                id: title_text_id,
                kind: UiNodeKind::Text,
                props: UiNodeProps::Text {
                    text: "Demo 7 - Telemetria de ponteiro por target".into(),
                    size: Some(22.0),
                    color: Some(UiColor {
                        r: 24,
                        g: 32,
                        b: 48,
                        a: 255,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(202),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(top_panel_id),
            node: UiNode {
                id: main_text_id,
                kind: UiNodeKind::Text,
                props: UiNodeProps::Text {
                    text: "Main target: aguardando eventos de ponteiro...".into(),
                    size: Some(16.0),
                    color: Some(UiColor {
                        r: 28,
                        g: 38,
                        b: 56,
                        a: 255,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(201),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(top_panel_id),
            node: UiNode {
                id: inner_text_id,
                kind: UiNodeKind::Text,
                props: UiNodeProps::Text {
                    text: "Inner target: aguardando eventos de ponteiro...".into(),
                    size: Some(16.0),
                    color: Some(UiColor {
                        r: 28,
                        g: 38,
                        b: 56,
                        a: 255,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(201),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(root_split_id),
            node: UiNode {
                id: bottom_panel_id,
                kind: UiNodeKind::Frame,
                props: UiNodeProps::Frame {
                    padding: Some(UiPadding {
                        left: 8.0,
                        top: 8.0,
                        right: 8.0,
                        bottom: 8.0,
                    }),
                    fill: Some(UiColor {
                        r: 230,
                        g: 233,
                        b: 239,
                        a: 255,
                    }),
                    stroke: None,
                    rounding: Some(6.0),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(202),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(bottom_panel_id),
            node: UiNode {
                id: bottom_panel_id + 1000,
                kind: UiNodeKind::Container,
                props: UiNodeProps::Container {
                    layout: UiLayout {
                        direction: UiLayoutDirection::Column,
                        align: Default::default(),
                        justify: Default::default(),
                        gap: 0.0,
                        columns: None,
                        wrap: false,
                        wrap_limit: None,
                    },
                    padding: None,
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                    scroll_x: false,
                    scroll_y: false,
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(202),
            },
            index: None,
        },
        UiOp::Add {
            parent: Some(bottom_panel_id + 1000),
            node: UiNode {
                id: viewport_id,
                kind: UiNodeKind::WidgetRealmViewport,
                props: UiNodeProps::WidgetRealmViewport {
                    target_id: inner_target_id,
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                },
                tooltip: None,
                context_menu: None,
                anim: None,
                display: None,
                visible: None,
                opacity: None,
                z_index: Some(202),
            },
            index: None,
        },
    ]
}

pub(super) fn format_vec2(value: Option<Vec2>) -> String {
    match value {
        Some(value) => format!("({:.1}, {:.1})", value.x, value.y),
        None => "-".into(),
    }
}
