use std::time::Duration;

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd, EngineEvent};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::events::UiEventKind;
use crate::core::ui::types::{
    UiColor, UiLayout, UiLayoutDirection, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPadding,
    UiPanelKind, UiSize,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, run_loop_with_events};

const DOC_ID: u32 = 98_100;
const TARGET_WINDOW: u64 = 98_000;

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
    let mut ui_version: u64 = 2;
    let mut click_count: u32 = 0;

    run_loop_with_events(
        ctx.window_id,
        None,
        |_total_ms, _delta_ms| Vec::new(),
        move |event| {
            let EngineEvent::Ui(ui_event) = event else {
                return false;
            };
            if ui_event.document_id != DOC_ID {
                return false;
            }

            let mut ops: Vec<UiOp> = Vec::new();
            if ui_event.node_id == 4 && ui_event.kind == UiEventKind::Click {
                click_count = click_count.saturating_add(1);
                ops.push(UiOp::Set {
                    node_id: 22,
                    props: UiNodeProps::Text {
                        text: format!("Clicks: {click_count}"),
                        size: None,
                        color: None,
                    },
                });
            }
            if ui_event.kind == UiEventKind::Changed {
                if let Some(value) = ui_event.label.clone() {
                    match ui_event.node_id {
                        5 => ops.push(UiOp::Set {
                            node_id: 5,
                            props: UiNodeProps::Checkbox {
                                label: "Checkbox".into(),
                                checked: value == "true",
                                enabled: Some(true),
                            },
                        }),
                        8 => ops.push(UiOp::Set {
                            node_id: 8,
                            props: UiNodeProps::Toggle {
                                label: "Toggle".into(),
                                value: value == "true",
                                enabled: Some(true),
                            },
                        }),
                        9 => {
                            if let Ok(parsed) = value.parse::<f64>() {
                                ops.push(UiOp::Set {
                                    node_id: 9,
                                    props: UiNodeProps::Slider {
                                        value: parsed,
                                        min: 0.0,
                                        max: 100.0,
                                        step: Some(1.0),
                                        label: Some("Slider".into()),
                                        enabled: Some(true),
                                    },
                                });
                            }
                        }
                        10 => {
                            if let Ok(parsed) = value.parse::<f64>() {
                                ops.push(UiOp::Set {
                                    node_id: 10,
                                    props: UiNodeProps::DragValue {
                                        value: parsed,
                                        speed: Some(0.1),
                                        min: Some(0.0),
                                        max: Some(10.0),
                                        prefix: Some("x=".into()),
                                        suffix: None,
                                        enabled: Some(true),
                                    },
                                });
                            }
                        }
                        12 => ops.push(UiOp::Set {
                            node_id: 12,
                            props: UiNodeProps::TextEdit {
                                value,
                                placeholder: Some("Digite...".into()),
                                multiline: Some(false),
                                password: Some(false),
                                char_limit: Some(64),
                                enabled: Some(true),
                            },
                        }),
                        13 => ops.push(UiOp::Set {
                            node_id: 13,
                            props: UiNodeProps::Input {
                                value,
                                placeholder: Some("Input".into()),
                                enabled: Some(true),
                            },
                        }),
                        14 => ops.push(UiOp::Set {
                            node_id: 14,
                            props: UiNodeProps::ComboBox {
                                label: "Combo".into(),
                                selected: value,
                                options: vec![
                                    "Opção A".into(),
                                    "Opção B".into(),
                                    "Opção C".into(),
                                ],
                                enabled: Some(true),
                            },
                        }),
                        _ => {}
                    }
                }
            }

            if ops.is_empty() {
                return false;
            }
            ui_version = ui_version.saturating_add(1);
            let cmds = vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                document_id: DOC_ID,
                version: ui_version,
                ops,
            })];
            let _ = send_commands(cmds);
            false
        },
    )
}

fn setup(ctx: DemoContext) -> u32 {
    drain_responses();
    let realm_ui = create_ui_realm(ctx.window_id, ctx.realm_id);

    let mut cmds = vec![
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id: TARGET_WINDOW,
            kind: TargetKind::Window,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: realm_ui,
            target_id: TARGET_WINDOW,
            layout: TargetLayerLayout {
                left: DimensionValue::Px(0.0),
                top: DimensionValue::Px(0.0),
                width: DimensionValue::Percent(100.0),
                height: DimensionValue::Percent(100.0),
                z_index: 1,
                blend_mode: 0,
                clip: None,
            },
            camera_id: None,
            environment_id: None,
        }),
        EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
            document_id: DOC_ID,
            realm_id: realm_ui,
            rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
            theme_id: None,
        }),
    ];

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
            size: Some(UiSize::default()),
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
            22,
            UiNodeKind::Text,
            UiNodeProps::Text {
                text: "Clicks: 0".into(),
                size: None,
                color: None,
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
    .map(|mut item| {
        item.tooltip = Some(format!("node {}", item.id));
        item.context_menu = Some(vec!["Ação A".into(), "Ação B".into()]);
        item
    })
    .collect();

    for item in nodes {
        ops.push(UiOp::Add {
            parent: Some(1),
            node: item,
            index: None,
        });
    }

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: DOC_ID,
        version: 1,
        ops,
    }));

    assert_eq!(send_commands(cmds), VulframResult::Success);
    wait_for_setup_batch();

    realm_ui
}

fn create_ui_realm(window_id: u32, old_realm_id: u32) -> u32 {
    let dispose_cmds = vec![EngineCmd::CmdRealmDispose(CmdRealmDisposeArgs {
        realm_id: old_realm_id,
    })];
    assert_eq!(send_commands(dispose_cmds), VulframResult::Success);
    let mut dispose_ok = false;
    for _ in 0..120 {
        for response in receive_responses() {
            if let CommandResponse::RealmDispose(result) = response.response {
                assert!(result.success, "[demo008:realm-dispose] {}", result.message);
                dispose_ok = true;
                break;
            }
        }
        if dispose_ok {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    assert!(dispose_ok, "[demo008:realm-dispose] missing response");

    let cmds = vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        output_surface_id: None,
        host_window_id: Some(window_id),
        importance: None,
        cache_policy: None,
        flags: None,
    })];
    assert_eq!(send_commands(cmds), VulframResult::Success);

    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "[demo008:realm-create] {}", result.message);
                if let Some(realm_id) = result.realm_id {
                    return realm_id;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }

    panic!("[demo008:realm-create] missing response");
}

fn drain_responses() {
    for _ in 0..16 {
        if receive_responses().is_empty() {
            break;
        }
    }
}

fn wait_for_setup_batch() {
    let mut got_target = false;
    let mut got_layer = false;
    let mut got_doc = false;
    let mut got_ops = false;

    for _ in 0..180 {
        for response in receive_responses() {
            match response.response {
                CommandResponse::TargetUpsert(result) => {
                    got_target = true;
                    assert!(result.success, "[demo008:target-upsert] {}", result.message);
                }
                CommandResponse::TargetLayerUpsert(result) => {
                    got_layer = true;
                    assert!(result.success, "[demo008:target-layer-upsert] {}", result.message);
                }
                CommandResponse::UiDocumentCreate(result) => {
                    got_doc = true;
                    assert!(result.success, "[demo008:ui-document-create] {}", result.message);
                }
                CommandResponse::UiApplyOps(result) => {
                    got_ops = true;
                    assert!(result.success, "[demo008:ui-apply-ops] {}", result.message);
                }
                _ => {}
            }
        }

        if got_target && got_layer && got_doc && got_ops {
            return;
        }

        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }

    panic!(
        "[demo008:setup] missing responses: target={} layer={} doc={} ops={}",
        got_target, got_layer, got_doc, got_ops
    );
}
