use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{
    UiColor, UiLayout, UiLayoutDirection, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPadding,
    UiPanelKind, UiSize,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, run_loop};

const TARGET_WINDOW: u64 = 98_000;
const DOC_ID: u32 = 98_100;

fn node(id: u32, kind: UiNodeKind, props: UiNodeProps) -> UiNode {
    UiNode {
        id,
        kind,
        props,
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    }
}

pub fn run(ctx: DemoContext) -> bool {
    let _realm_ui = setup(ctx);
    run_loop(ctx.window_id, None, |_total_ms, _delta_ms| Vec::new())
}

fn setup(ctx: DemoContext) -> u32 {
    let _ = send_commands(vec![EngineCmd::CmdRealmDispose(CmdRealmDisposeArgs {
        realm_id: ctx.realm_id,
    })]);
    let _ = receive_responses();

    let _ = send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        output_surface_id: None,
        host_window_id: Some(ctx.window_id),
        importance: None,
        cache_policy: None,
        flags: None,
    })]);

    let mut realm_ui = 0;
    for response in receive_responses() {
        if let CommandResponse::RealmCreate(result) = response.response
            && result.success
        {
            realm_ui = result.realm_id.unwrap_or(0);
        }
    }

    assert_eq!(
        send_commands(vec![EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id: TARGET_WINDOW,
            kind: TargetKind::Window,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        })]),
        VulframResult::Success
    );
    assert_eq!(
        send_commands(vec![EngineCmd::CmdTargetLayerUpsert(
            CmdTargetLayerUpsertArgs {
                realm_id: realm_ui,
                target_id: TARGET_WINDOW,
                layout: TargetLayerLayout {
                    left: DimensionValue::Px(0.0),
                    top: DimensionValue::Px(0.0),
                    width: DimensionValue::Percent(100.0),
                    height: DimensionValue::Percent(100.0),
                    z_index: 5,
                    blend_mode: 0,
                    clip: None,
                },
                camera_id: None,
                environment_id: None,
            }
        )]),
        VulframResult::Success
    );
    let _ = receive_responses();

    let _ = send_commands(vec![EngineCmd::CmdUiDocumentCreate(
        CmdUiDocumentCreateArgs {
            document_id: DOC_ID,
            realm_id: realm_ui,
            rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
            theme_id: None,
        },
    )]);
    let _ = receive_responses();

    let root = node(
        1,
        UiNodeKind::Container,
        UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 6.0,
                ..Default::default()
            },
            padding: Some(UiPadding {
                left: 16.0,
                top: 12.0,
                right: 16.0,
                bottom: 12.0,
            }),
            size: Some(UiSize {
                width: crate::core::ui::types::UiLength::Fill,
                height: crate::core::ui::types::UiLength::Fill,
            }),
            scroll_x: false,
            scroll_y: true,
        },
    );

    let mut ops = vec![UiOp::Add {
        parent: None,
        node: root,
        index: None,
    }];
    let nodes: Vec<UiNode> = vec![
        node(
            2,
            UiNodeKind::Text,
            UiNodeProps::Text {
                text: "Demo 008: UI Widgets Showcase".into(),
                size: Some(28.0),
                color: Some(UiColor {
                    r: 230,
                    g: 234,
                    b: 245,
                    a: 255,
                }),
            },
        ),
        node(
            3,
            UiNodeKind::RichText,
            UiNodeProps::RichText {
                text: "Botões, inputs, seleção, menus e estados.".into(),
                size: Some(14.0),
                color: Some(UiColor {
                    r: 190,
                    g: 198,
                    b: 220,
                    a: 255,
                }),
                strong: Some(false),
                italics: Some(true),
                underline: Some(false),
                strikethrough: Some(false),
                monospace: Some(false),
            },
        ),
        node(
            4,
            UiNodeKind::Button,
            UiNodeProps::Button {
                label: "Button".into(),
                enabled: Some(true),
            },
        ),
        node(
            5,
            UiNodeKind::Checkbox,
            UiNodeProps::Checkbox {
                label: "Checkbox".into(),
                checked: true,
                enabled: Some(true),
            },
        ),
        node(
            6,
            UiNodeKind::Radio,
            UiNodeProps::Radio {
                label: "Radio".into(),
                selected: false,
                enabled: Some(true),
            },
        ),
        node(
            7,
            UiNodeKind::SelectableLabel,
            UiNodeProps::SelectableLabel {
                label: "Selectable Label".into(),
                selected: true,
                enabled: Some(true),
            },
        ),
        node(
            8,
            UiNodeKind::Toggle,
            UiNodeProps::Toggle {
                label: "Toggle".into(),
                value: true,
                enabled: Some(true),
            },
        ),
        node(
            9,
            UiNodeKind::Slider,
            UiNodeProps::Slider {
                value: 35.0,
                min: 0.0,
                max: 100.0,
                step: Some(1.0),
                label: Some("Slider".into()),
                enabled: Some(true),
            },
        ),
        node(
            10,
            UiNodeKind::DragValue,
            UiNodeProps::DragValue {
                value: 5.0,
                speed: Some(0.1),
                min: Some(0.0),
                max: Some(10.0),
                prefix: Some("x=".into()),
                suffix: None,
                enabled: Some(true),
            },
        ),
        node(
            11,
            UiNodeKind::ProgressBar,
            UiNodeProps::ProgressBar {
                value: 0.62,
                text: Some("Progress".into()),
                animate: Some(true),
                show_percentage: Some(true),
            },
        ),
        node(
            12,
            UiNodeKind::TextEdit,
            UiNodeProps::TextEdit {
                value: "Texto editável".into(),
                placeholder: Some("Digite...".into()),
                multiline: Some(false),
                password: Some(false),
                char_limit: Some(64),
                enabled: Some(true),
            },
        ),
        node(
            13,
            UiNodeKind::Input,
            UiNodeProps::Input {
                value: "Input simples".into(),
                placeholder: Some("Input".into()),
                enabled: Some(true),
            },
        ),
        node(
            14,
            UiNodeKind::ComboBox,
            UiNodeProps::ComboBox {
                label: "Combo".into(),
                selected: "Opção B".into(),
                options: vec!["Opção A".into(), "Opção B".into(), "Opção C".into()],
                enabled: Some(true),
            },
        ),
        node(
            15,
            UiNodeKind::MenuButton,
            UiNodeProps::MenuButton {
                label: "Menu".into(),
                enabled: Some(true),
            },
        ),
        node(
            16,
            UiNodeKind::CollapsingHeader,
            UiNodeProps::CollapsingHeader {
                label: "Detalhes".into(),
                open: Some(true),
                enabled: Some(true),
            },
        ),
        node(
            17,
            UiNodeKind::Link,
            UiNodeProps::Link {
                label: "Link interno".into(),
                enabled: Some(true),
            },
        ),
        node(
            18,
            UiNodeKind::Hyperlink,
            UiNodeProps::Hyperlink {
                label: "Site do egui".into(),
                url: "https://github.com/emilk/egui".into(),
                enabled: Some(true),
            },
        ),
        node(
            19,
            UiNodeKind::Spinner,
            UiNodeProps::Spinner { size: Some(18.0) },
        ),
        node(20, UiNodeKind::Separator, UiNodeProps::Separator),
        node(
            21,
            UiNodeKind::Panel,
            UiNodeProps::Panel {
                kind: UiPanelKind::Top,
                resizable: Some(false),
                size: Some(UiSize {
                    width: crate::core::ui::types::UiLength::Fill,
                    height: crate::core::ui::types::UiLength::Px(90.0),
                }),
                min_size: None,
                max_size: None,
            },
        ),
    ]
    .into_iter()
    .map(|mut node| {
        node.tooltip = Some(format!("node {}", node.id));
        node.context_menu = Some(vec!["Ação A".into(), "Ação B".into()]);
        node
    })
    .collect();

    for node in nodes {
        ops.push(UiOp::Add {
            parent: Some(1),
            node,
            index: None,
        });
    }

    let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: DOC_ID,
        version: 1,
        ops,
    })]);
    let _ = receive_responses();

    realm_ui
}
