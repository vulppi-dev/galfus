use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{
    UiLayout, UiLayoutDirection, UiNode, UiNodeKind, UiNodeProps, UiOp, UiSize, UiSplitDirection,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, run_loop};
use std::time::Duration;

const TARGET_WINDOW: u64 = 99_000;
const DOC_ID: u32 = 99_100;

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
    wait_for_setup_responses(2);

    let _ = send_commands(vec![EngineCmd::CmdUiDocumentCreate(
        CmdUiDocumentCreateArgs {
            document_id: DOC_ID,
            realm_id: realm_ui,
            rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
            theme_id: None,
        },
    )]);
    wait_for_setup_responses(1);

    let ops = vec![
        UiOp::Add {
            parent: None,
            node: node(
                1,
                UiNodeKind::SplitPane,
                UiNodeProps::SplitPane {
                    direction: UiSplitDirection::Horizontal,
                    ratio: Some(0.25),
                    resizable: Some(true),
                    min_a: Some(180.0),
                    max_a: Some(500.0),
                    min_b: Some(280.0),
                    max_b: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                2,
                UiNodeKind::Container,
                UiNodeProps::Container {
                    layout: UiLayout {
                        direction: UiLayoutDirection::Column,
                        gap: 8.0,
                        ..Default::default()
                    },
                    padding: None,
                    size: Some(UiSize::default()),
                    scroll_x: false,
                    scroll_y: true,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(2),
            node: node(
                20,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Demo 009: Panels + Splitter".into(),
                    size: Some(20.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(2),
            node: node(
                21,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Painel lateral (explorer)".into(),
                    size: None,
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                3,
                UiNodeKind::SplitPane,
                UiNodeProps::SplitPane {
                    direction: UiSplitDirection::Vertical,
                    ratio: Some(0.72),
                    resizable: Some(true),
                    min_a: Some(180.0),
                    max_a: None,
                    min_b: Some(90.0),
                    max_b: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(3),
            node: node(
                4,
                UiNodeKind::Container,
                UiNodeProps::Container {
                    layout: UiLayout {
                        direction: UiLayoutDirection::Column,
                        gap: 10.0,
                        ..Default::default()
                    },
                    padding: None,
                    size: Some(UiSize::default()),
                    scroll_x: false,
                    scroll_y: false,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(4),
            node: node(
                40,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Área central (dock básico)".into(),
                    size: Some(18.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(4),
            node: node(
                41,
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Ajuste os divisores para validar pointer-resize.".into(),
                    size: None,
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(3),
            node: node(
                5,
                UiNodeKind::Container,
                UiNodeProps::Container {
                    layout: UiLayout {
                        direction: UiLayoutDirection::Row,
                        gap: 12.0,
                        ..Default::default()
                    },
                    padding: None,
                    size: Some(UiSize::default()),
                    scroll_x: false,
                    scroll_y: false,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(5),
            node: node(
                50,
                UiNodeKind::Button,
                UiNodeProps::Button {
                    label: "Console".into(),
                    enabled: Some(true),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(5),
            node: node(
                51,
                UiNodeKind::Button,
                UiNodeProps::Button {
                    label: "Output".into(),
                    enabled: Some(true),
                },
            ),
            index: None,
        },
    ];
    assert_eq!(
        send_commands(vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            document_id: DOC_ID,
            version: 1,
            ops,
        })]),
        VulframResult::Success
    );
    wait_for_setup_responses(1);

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
                assert!(result.success, "[demo009:realm-dispose] {}", result.message);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo009:realm-dispose] missing response");
}

fn wait_for_realm_create() -> u32 {
    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "[demo009:realm-create] {}", result.message);
                if let Some(realm_id) = result.realm_id {
                    return realm_id;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo009:realm-create] missing response");
}

fn wait_for_setup_responses(expected: usize) {
    let mut count = 0usize;
    for _ in 0..180 {
        for response in receive_responses() {
            match response.response {
                CommandResponse::TargetUpsert(result) => {
                    assert!(result.success, "[demo009:target-upsert] {}", result.message);
                    count += 1;
                }
                CommandResponse::TargetLayerUpsert(result) => {
                    assert!(
                        result.success,
                        "[demo009:target-layer-upsert] {}",
                        result.message
                    );
                    count += 1;
                }
                CommandResponse::UiDocumentCreate(result) => {
                    assert!(
                        result.success,
                        "[demo009:ui-document-create] {}",
                        result.message
                    );
                    count += 1;
                }
                CommandResponse::UiApplyOps(result) => {
                    assert!(result.success, "[demo009:ui-apply-ops] {}", result.message);
                    count += 1;
                }
                _ => {}
            }
        }
        if count >= expected {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo009:setup] missing responses");
}
