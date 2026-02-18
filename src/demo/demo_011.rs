use std::time::Duration;

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{
    UiColor, UiLength, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPadding, UiSize, UiStroke,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, run_loop};

const TARGET_WINDOW: u64 = 101_000;
const DOC_ID: u32 = 101_100;

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
    drain_responses();
    assert_eq!(
        send_commands(vec![EngineCmd::CmdRealmDispose(CmdRealmDisposeArgs {
            realm_id: ctx.realm_id,
        })]),
        VulframResult::Success
    );
    wait_for_realm_dispose();

    assert_eq!(
        send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(ctx.window_id),
            importance: None,
            cache_policy: None,
            flags: None,
        })]),
        VulframResult::Success
    );
    let realm_ui = wait_for_realm_create();

    let ops = vec![
        UiOp::Add {
            parent: None,
            node: node(
                1,
                UiNodeKind::Container,
                UiNodeProps::Container {
                    layout: Default::default(),
                    padding: Some(UiPadding {
                        left: 12.0,
                        top: 12.0,
                        right: 12.0,
                        bottom: 12.0,
                    }),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                    scroll_x: false,
                    scroll_y: true,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                2,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Demo 011: Window/Area/Frame/Scroll/Grid/Popup/Modal/Resize/Spacer"
                        .into(),
                    size: Some(18.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                3,
                UiNodeKind::Spacer,
                UiNodeProps::Spacer {
                    width: Some(0.0),
                    height: Some(8.0),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                10,
                UiNodeKind::Frame,
                UiNodeProps::Frame {
                    padding: Some(UiPadding {
                        left: 10.0,
                        top: 10.0,
                        right: 10.0,
                        bottom: 10.0,
                    }),
                    fill: Some(UiColor {
                        r: 50,
                        g: 63,
                        b: 86,
                        a: 255,
                    }),
                    stroke: Some(UiStroke {
                        width: 1.0,
                        color: UiColor {
                            r: 130,
                            g: 160,
                            b: 210,
                            a: 255,
                        },
                    }),
                    rounding: Some(8.0),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Px(240.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(10),
            node: node(
                11,
                UiNodeKind::ScrollArea,
                UiNodeProps::ScrollArea {
                    scroll_x: false,
                    scroll_y: true,
                    auto_shrink: Some(false),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(11),
            node: node(
                12,
                UiNodeKind::Grid,
                UiNodeProps::Grid {
                    columns: Some(3),
                    striped: Some(true),
                    min_col_width: Some(120.0),
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Auto,
                    }),
                },
            ),
            index: None,
        },
    ];

    let mut ops = ops;
    for i in 0..18 {
        ops.push(UiOp::Add {
            parent: Some(12),
            node: node(
                100 + i,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: format!("cell {}", i + 1),
                    size: None,
                    color: None,
                },
            ),
            index: None,
        });
    }

    ops.extend([
        UiOp::Add {
            parent: Some(1),
            node: node(
                20,
                UiNodeKind::Resize,
                UiNodeProps::Resize {
                    size: Some(UiSize {
                        width: UiLength::Px(280.0),
                        height: UiLength::Px(120.0),
                    }),
                    min_size: Some(UiSize {
                        width: UiLength::Px(160.0),
                        height: UiLength::Px(80.0),
                    }),
                    max_size: Some(UiSize {
                        width: UiLength::Px(500.0),
                        height: UiLength::Px(220.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(20),
            node: node(
                21,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Resize me".into(),
                    size: Some(16.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                30,
                UiNodeKind::Window,
                UiNodeProps::Window {
                    title: "Floating Window".into(),
                    open: Some(true),
                    movable: Some(true),
                    resizable: Some(true),
                    collapsible: Some(true),
                    anchored: Some(crate::core::ui::types::UiWindowAnchor { x: 740.0, y: 40.0 }),
                    size: Some(UiSize {
                        width: UiLength::Px(280.0),
                        height: UiLength::Px(160.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(30),
            node: node(
                31,
                UiNodeKind::Tooltip,
                UiNodeProps::Tooltip {
                    text: "Tooltip node text".into(),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                40,
                UiNodeKind::Area,
                UiNodeProps::Area {
                    label: Some("Floating Area".into()),
                    x: Some(760.0),
                    y: Some(260.0),
                    draggable: Some(true),
                    size: Some(UiSize {
                        width: UiLength::Px(230.0),
                        height: UiLength::Px(100.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(40),
            node: node(
                41,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Draggable Area".into(),
                    size: Some(15.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                50,
                UiNodeKind::Popup,
                UiNodeProps::Popup {
                    title: Some("Popup Open".into()),
                    open: Some(true),
                    size: Some(UiSize {
                        width: UiLength::Px(260.0),
                        height: UiLength::Px(120.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(50),
            node: node(
                51,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Popup content".into(),
                    size: None,
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                60,
                UiNodeKind::Modal,
                UiNodeProps::Modal {
                    title: "Modal Open".into(),
                    open: Some(true),
                    size: Some(UiSize {
                        width: UiLength::Px(340.0),
                        height: UiLength::Px(180.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(60),
            node: node(
                61,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Modal content".into(),
                    size: Some(16.0),
                    color: None,
                },
            ),
            index: None,
        },
    ]);

    let cmds = vec![
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
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            document_id: DOC_ID,
            version: 1,
            ops,
        }),
    ];
    assert_eq!(send_commands(cmds), VulframResult::Success);
    wait_for_setup_batch();
    realm_ui
}

fn drain_responses() {
    for _ in 0..16 {
        if receive_responses().is_empty() {
            break;
        }
    }
}

fn wait_for_realm_dispose() {
    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::RealmDispose(result) = response.response {
                assert!(result.success, "[demo011:realm-dispose] {}", result.message);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo011:realm-dispose] missing response");
}

fn wait_for_realm_create() -> u32 {
    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "[demo011:realm-create] {}", result.message);
                if let Some(realm_id) = result.realm_id {
                    return realm_id;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo011:realm-create] missing response");
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
                    assert!(result.success, "[demo011:target-upsert] {}", result.message);
                }
                CommandResponse::TargetLayerUpsert(result) => {
                    got_layer = true;
                    assert!(result.success, "[demo011:target-layer-upsert] {}", result.message);
                }
                CommandResponse::UiDocumentCreate(result) => {
                    got_doc = true;
                    assert!(result.success, "[demo011:ui-document-create] {}", result.message);
                }
                CommandResponse::UiApplyOps(result) => {
                    got_ops = true;
                    assert!(result.success, "[demo011:ui-apply-ops] {}", result.message);
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
    panic!("[demo011:setup] missing responses");
}
