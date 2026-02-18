use std::time::Duration;

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, CmdUiImageCreateFromBufferArgs};
use crate::core::ui::types::{
    UiColor, UiImageSource, UiLength, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPaintOp, UiPaintStroke,
    UiSize,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, load_texture_bytes, run_loop, upload_texture_bytes};

const TARGET_WINDOW: u64 = 102_000;
const DOC_ID: u32 = 102_100;
const UI_IMAGE_ID: u32 = 102_200;
const UI_IMAGE_BUFFER_ID: u64 = 102_201;

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

    let image_bytes = load_texture_bytes("assets/icon.png");
    upload_texture_bytes(&image_bytes, UI_IMAGE_BUFFER_ID);

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
        EngineCmd::CmdUiImageCreateFromBuffer(CmdUiImageCreateFromBufferArgs {
            image_id: UI_IMAGE_ID,
            buffer_id: UI_IMAGE_BUFFER_ID,
            label: Some("demo012-ui-image".into()),
        }),
    ];
    assert_eq!(send_commands(cmds), VulframResult::Success);
    wait_for_setup_batch();

    let scene_stroke = UiPaintStroke {
        width: 2.0,
        color: UiColor {
            r: 110,
            g: 220,
            b: 250,
            a: 255,
        },
        join: None,
        cap: None,
    };
    let ops = vec![
        UiOp::Add {
            parent: None,
            node: node(
                1,
                UiNodeKind::Container,
                UiNodeProps::Container {
                    layout: Default::default(),
                    padding: None,
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
                    text: "Demo 012: Scene + Image + ImageButton".into(),
                    size: Some(18.0),
                    color: None,
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                10,
                UiNodeKind::Scene,
                UiNodeProps::Scene {
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Px(320.0),
                    }),
                    zoom_min: Some(0.25),
                    zoom_max: Some(5.0),
                    pan_enabled: Some(true),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(10),
            node: node(
                11,
                UiNodeKind::Canvas,
                UiNodeProps::Canvas {
                    ops: vec![
                        UiPaintOp::RectFilled {
                            min: glam::vec2(30.0, 30.0),
                            max: glam::vec2(680.0, 280.0),
                            rounding: Some(8.0),
                            fill: UiColor {
                                r: 28,
                                g: 40,
                                b: 58,
                                a: 255,
                            },
                        },
                        UiPaintOp::QuadraticBezier {
                            from: glam::vec2(60.0, 240.0),
                            ctrl: glam::vec2(340.0, 40.0),
                            to: glam::vec2(640.0, 220.0),
                            steps: Some(64),
                            stroke: scene_stroke,
                        },
                    ],
                    size: Some(UiSize {
                        width: UiLength::Fill,
                        height: UiLength::Fill,
                    }),
                    clip: Some(true),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                20,
                UiNodeKind::Image,
                UiNodeProps::Image {
                    source: UiImageSource::UiImage(UI_IMAGE_ID),
                    size: Some(UiSize {
                        width: UiLength::Px(128.0),
                        height: UiLength::Px(128.0),
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: Some(1),
            node: node(
                21,
                UiNodeKind::ImageButton,
                UiNodeProps::ImageButton {
                    source: UiImageSource::UiImage(UI_IMAGE_ID),
                    size: Some(UiSize {
                        width: UiLength::Px(96.0),
                        height: UiLength::Px(96.0),
                    }),
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
    wait_for_apply_ops();

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
                assert!(result.success, "[demo012:realm-dispose] {}", result.message);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo012:realm-dispose] missing response");
}

fn wait_for_realm_create() -> u32 {
    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "[demo012:realm-create] {}", result.message);
                if let Some(realm_id) = result.realm_id {
                    return realm_id;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo012:realm-create] missing response");
}

fn wait_for_setup_batch() {
    let mut got_target = false;
    let mut got_layer = false;
    let mut got_doc = false;
    let mut got_image = false;

    for _ in 0..180 {
        for response in receive_responses() {
            match response.response {
                CommandResponse::TargetUpsert(result) => {
                    got_target = true;
                    assert!(result.success, "[demo012:target-upsert] {}", result.message);
                }
                CommandResponse::TargetLayerUpsert(result) => {
                    got_layer = true;
                    assert!(result.success, "[demo012:target-layer-upsert] {}", result.message);
                }
                CommandResponse::UiDocumentCreate(result) => {
                    got_doc = true;
                    assert!(result.success, "[demo012:ui-document-create] {}", result.message);
                }
                CommandResponse::UiImageCreateFromBuffer(result) => {
                    got_image = true;
                    assert!(result.success, "[demo012:ui-image-create] {}", result.message);
                }
                _ => {}
            }
        }
        if got_target && got_layer && got_doc && got_image {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo012:setup] missing responses");
}

fn wait_for_apply_ops() {
    for _ in 0..180 {
        for response in receive_responses() {
            if let CommandResponse::UiApplyOps(result) = response.response {
                assert!(result.success, "[demo012:ui-apply-ops] {}", result.message);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo012:ui-apply-ops] missing response");
}
