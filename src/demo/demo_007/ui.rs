use super::maps::Demo007TargetIds;
use super::setup::Demo007Ids;
use crate::core::cmd::EngineCmd;
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::types::{
    UiLayout, UiLayoutDirection, UiLength, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPadding,
    UiSize, UiThemeValue,
};

pub fn build_ui_cmds(ids: Demo007Ids, targets: Demo007TargetIds, realm_ui: u32) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    cmds.push(EngineCmd::CmdUiThemeDefine(CmdUiThemeDefineArgs {
        theme_id: 7,
        version: None,
        data: std::collections::HashMap::from([
            ("fontSize".into(), UiThemeValue::Float(16.0)),
            ("textColor".into(), UiThemeValue::String("#F0F0F0".into())),
            ("panelFill".into(), UiThemeValue::String("#121417".into())),
        ]),
        font_data: std::collections::HashMap::new(),
        font_families: std::collections::HashMap::new(),
    }));

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_document_id,
        realm_id: realm_ui,
        rect: glam::Vec4::new(0.0, 0.0, 1280.0, 720.0),
        theme_id: Some(7),
    }));

    let root = UiNode {
        id: ids.ui_root_id,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 12.0,
                ..Default::default()
            },
            padding: Some(UiPadding {
                left: 16.0,
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
            }),
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
        opacity: Some(1.0),
        z_index: None,
    };

    let title = UiNode {
        id: ids.ui_title_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "Demo 007: 4x Widget Realm Viewport".into(),
            size: Some(22.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let grid = UiNode {
        id: ids.ui_grid_id,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 12.0,
                ..Default::default()
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
        z_index: None,
    };

    let row_nodes = [7450, 7460];
    let widgets = [
        (row_nodes[0], 7410, 7411, "Camera A", targets.widget_view_a),
        (row_nodes[0], 7420, 7421, "Camera B", targets.widget_view_b),
        (row_nodes[1], 7430, 7431, "Camera C", targets.widget_view_c),
        (row_nodes[1], 7440, 7441, "Camera D", targets.widget_view_d),
    ];

    let mut ops = vec![
        UiOp::Add {
            parent: None,
            node: root,
            index: None,
        },
        UiOp::Add {
            parent: Some(ids.ui_root_id),
            node: title,
            index: None,
        },
        UiOp::Add {
            parent: Some(ids.ui_root_id),
            node: grid,
            index: None,
        },
    ];

    for row_id in row_nodes {
        let row = UiNode {
            id: row_id,
            kind: UiNodeKind::Container,
            props: UiNodeProps::Container {
                layout: UiLayout {
                    direction: UiLayoutDirection::Row,
                    gap: 12.0,
                    ..Default::default()
                },
                padding: None,
                size: Some(UiSize {
                    width: UiLength::Fill,
                    height: UiLength::Px(300.0),
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
            z_index: None,
        };
        ops.push(UiOp::Add {
            parent: Some(ids.ui_grid_id),
            node: row,
            index: None,
        });
    }

    for (row_id, container_id, node_id, label, target_id) in widgets {
        let panel = UiNode {
            id: container_id,
            kind: UiNodeKind::Container,
            props: UiNodeProps::Container {
                layout: UiLayout {
                    direction: UiLayoutDirection::Column,
                    gap: 6.0,
                    ..Default::default()
                },
                padding: Some(UiPadding {
                    left: 8.0,
                    top: 8.0,
                    right: 8.0,
                    bottom: 8.0,
                }),
                size: Some(UiSize {
                    width: UiLength::Auto,
                    height: UiLength::Px(300.0),
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
            z_index: None,
        };
        let label = UiNode {
            id: node_id,
            kind: UiNodeKind::Text,
            props: UiNodeProps::Text {
                text: label.into(),
                size: Some(14.0),
                color: None,
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index: None,
        };
        let viewport = UiNode {
            id: node_id + 1,
            kind: UiNodeKind::WidgetRealmViewport,
            props: UiNodeProps::WidgetRealmViewport {
                target_id,
                size: Some(UiSize {
                    width: UiLength::Px(560.0),
                    height: UiLength::Px(260.0),
                }),
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index: None,
        };

        ops.push(UiOp::Add {
            parent: Some(row_id),
            node: panel,
            index: None,
        });
        ops.push(UiOp::Add {
            parent: Some(container_id),
            node: label,
            index: None,
        });
        ops.push(UiOp::Add {
            parent: Some(container_id),
            node: viewport,
            index: None,
        });
    }

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: ids.ui_document_id,
        version: 1,
        ops,
    }));

    cmds
}
