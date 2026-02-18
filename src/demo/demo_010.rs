use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{
    UiColor, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPaintOp, UiPaintStroke,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, run_loop};

const TARGET_WINDOW: u64 = 100_000;
const DOC_ID: u32 = 100_100;

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

    let stroke_main = UiPaintStroke {
        width: 2.0,
        color: UiColor {
            r: 130,
            g: 210,
            b: 255,
            a: 255,
        },
        join: None,
        cap: None,
    };
    let stroke_alt = UiPaintStroke {
        width: 1.6,
        color: UiColor {
            r: 255,
            g: 190,
            b: 140,
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
                UiNodeKind::Text,
                UiNodeProps::Text {
                    text: "Demo 010: Painter/Path".into(),
                    size: Some(30.0),
                    color: Some(UiColor {
                        r: 230,
                        g: 235,
                        b: 245,
                        a: 255,
                    }),
                },
            ),
            index: None,
        },
        UiOp::Add {
            parent: None,
            node: node(
                2,
                UiNodeKind::Canvas,
                UiNodeProps::Canvas {
                    ops: vec![
                        UiPaintOp::RectFilled {
                            min: glam::vec2(10.0, 10.0),
                            max: glam::vec2(950.0, 620.0),
                            rounding: Some(10.0),
                            fill: UiColor {
                                r: 20,
                                g: 28,
                                b: 44,
                                a: 255,
                            },
                        },
                        UiPaintOp::LineSegment {
                            from: glam::vec2(60.0, 580.0),
                            to: glam::vec2(900.0, 120.0),
                            stroke: stroke_main.clone(),
                        },
                        UiPaintOp::Circle {
                            center: glam::vec2(300.0, 280.0),
                            radius: 90.0,
                            stroke: stroke_alt.clone(),
                        },
                        UiPaintOp::CircleFilled {
                            center: glam::vec2(510.0, 260.0),
                            radius: 62.0,
                            fill: UiColor {
                                r: 90,
                                g: 150,
                                b: 245,
                                a: 220,
                            },
                        },
                        UiPaintOp::QuadraticBezier {
                            from: glam::vec2(80.0, 460.0),
                            ctrl: glam::vec2(510.0, 40.0),
                            to: glam::vec2(850.0, 430.0),
                            steps: Some(60),
                            stroke: stroke_main.clone(),
                        },
                        UiPaintOp::CubicBezier {
                            from: glam::vec2(120.0, 500.0),
                            ctrl1: glam::vec2(220.0, 200.0),
                            ctrl2: glam::vec2(760.0, 620.0),
                            to: glam::vec2(880.0, 260.0),
                            steps: Some(80),
                            stroke: stroke_alt.clone(),
                        },
                        UiPaintOp::ConvexPolygon {
                            points: vec![
                                glam::vec2(650.0, 520.0),
                                glam::vec2(790.0, 560.0),
                                glam::vec2(860.0, 450.0),
                                glam::vec2(740.0, 390.0),
                            ],
                            fill: UiColor {
                                r: 120,
                                g: 80,
                                b: 210,
                                a: 220,
                            },
                            stroke: Some(stroke_main.clone()),
                        },
                        UiPaintOp::Text {
                            position: glam::vec2(50.0, 45.0),
                            text: "Canvas com primitives e paths".into(),
                            size: Some(26.0),
                            color: UiColor {
                                r: 235,
                                g: 235,
                                b: 245,
                                a: 255,
                            },
                            align: None,
                        },
                    ],
                    size: Some(crate::core::ui::types::UiSize {
                        width: crate::core::ui::types::UiLength::Fill,
                        height: crate::core::ui::types::UiLength::Fill,
                    }),
                    clip: Some(true),
                },
            ),
            index: None,
        },
    ];

    let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: DOC_ID,
        version: 1,
        ops,
    })]);
    let _ = receive_responses();

    realm_ui
}
