use std::time::Duration;

use glam::{Mat4, Vec3, Vec4};

use super::pointer_listener_lab_ui::{build_ui_tree, format_vec2};
use super::{DemoContext, DemoIds, create_ui_realm};
use crate::core;
use crate::core::cmd::CmdEnvironmentUpsertArgs;
use crate::core::cmd::{CmdModelUpsertArgs, CommandResponse, EngineCmd, EngineEvent};
use crate::core::input::listeners::CmdInputTargetListenerUpsertArgs;
use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, EnvironmentConfig, SkyboxConfig, SkyboxMode,
};
use crate::core::system::SystemEvent;
use crate::core::target::{
    DimensionValue, TargetKind, TargetLayerLayout,
    cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs},
};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use crate::core::ui::types::{UiColor, UiNodeProps, UiOp};
use crate::demo::commands::{
    create_ambient_light_cmd, create_camera_cmd, create_point_light_cmd,
    create_standard_material_cmd,
};
use crate::demo::hud::FpsHud;
use crate::demo::io::{receive_events, receive_responses, send_commands};
use crate::demo::loop_utils::run_loop_with_events;

pub(super) fn run_demo_7_pointer_listener_lab(ctx: DemoContext) -> bool {
    let ids = DemoIds::from_number(7);
    let Some(left_realm_id) = create_realm(RealmKindDto::ThreeD) else {
        return false;
    };
    let Some(ui_realm_id) = create_ui_realm(ctx.window_id) else {
        return false;
    };
    let Some(inner_realm_id) = create_realm(RealmKindDto::ThreeD) else {
        return false;
    };

    let left_target_id = ids.target_id + 7_000;
    let inner_target_id = ids.target_id + 7_002;
    let doc_id = ids.ui_doc_extra + 700;
    let root_split_id = ids.ui_node_extra + 700;
    let top_panel_id = root_split_id + 1;
    let bottom_panel_id = root_split_id + 2;
    let title_text_id = root_split_id + 3;
    let main_text_id = root_split_id + 4;
    let inner_text_id = root_split_id + 5;
    let viewport_id = root_split_id + 6;

    let left_camera_id = ids.camera_id + 700;
    let inner_camera_id = ids.camera_id + 701;
    let left_geometry_id = ids.geometry_id + 700;
    let inner_geometry_id = ids.geometry_id + 701;
    let left_material_id = ids.material_id + 700;
    let inner_material_id = ids.material_id + 701;
    let left_env_id = ids.env_id + 700;
    let inner_env_id = ids.env_id + 701;
    let left_model_id = ids.model_id + 700;
    let inner_model_id = ids.model_id + 701;
    let left_light_id = ids.light_id + 700;
    let inner_light_id = ids.light_id + 701;

    let main_listener_id = 970_001_u64;
    let inner_listener_id = 970_002_u64;

    let mut hud = FpsHud::new(7);
    let mut setup_cmds: Vec<EngineCmd> = vec![
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id: left_target_id,
            kind: TargetKind::Window,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: left_realm_id,
            target_id: left_target_id,
            layout: TargetLayerLayout {
                left: DimensionValue::Percent(0.0),
                top: DimensionValue::Percent(0.0),
                width: DimensionValue::Percent(50.0),
                height: DimensionValue::Percent(100.0),
                z_index: 1,
                blend_mode: 0,
                clip: None,
            },
            camera_id: Some(left_camera_id),
            environment_id: Some(left_env_id),
        }),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: ui_realm_id,
            target_id: left_target_id,
            layout: TargetLayerLayout {
                left: DimensionValue::Percent(50.0),
                top: DimensionValue::Percent(0.0),
                width: DimensionValue::Percent(50.0),
                height: DimensionValue::Percent(100.0),
                z_index: 2,
                blend_mode: 0,
                clip: None,
            },
            camera_id: None,
            environment_id: None,
        }),
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id: inner_target_id,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(1),
        }),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: inner_realm_id,
            target_id: inner_target_id,
            layout: TargetLayerLayout {
                left: DimensionValue::Percent(50.0),
                top: DimensionValue::Percent(45.0),
                width: DimensionValue::Percent(50.0),
                height: DimensionValue::Percent(55.0),
                z_index: 3,
                blend_mode: 0,
                clip: None,
            },
            camera_id: Some(inner_camera_id),
            environment_id: Some(inner_env_id),
        }),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: left_env_id,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.05, 0.09, 0.16, 1.0),
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: inner_env_id,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.10, 0.07, 0.12, 1.0),
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdPrimitiveGeometryCreate(
            crate::core::resources::CmdPrimitiveGeometryCreateArgs {
                geometry_id: left_geometry_id,
                label: Some("Demo7 Left Cube".into()),
                shape: crate::core::resources::PrimitiveShape::Cube,
                options: None,
            },
        ),
        EngineCmd::CmdPrimitiveGeometryCreate(
            crate::core::resources::CmdPrimitiveGeometryCreateArgs {
                geometry_id: inner_geometry_id,
                label: Some("Demo7 Inner Cube".into()),
                shape: crate::core::resources::PrimitiveShape::Cube,
                options: None,
            },
        ),
        create_camera_cmd(
            left_realm_id,
            left_camera_id,
            "Demo7 Left Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_camera_cmd(
            inner_realm_id,
            inner_camera_id,
            "Demo7 Inner Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 2.2, 4.5), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(left_realm_id, left_light_id, Vec4::new(2.5, 4.5, 2.2, 1.0)),
        create_ambient_light_cmd(
            left_realm_id,
            left_light_id + 1,
            Vec4::new(0.22, 0.28, 0.38, 1.0),
            0.55,
        ),
        create_point_light_cmd(
            inner_realm_id,
            inner_light_id,
            Vec4::new(1.8, 3.4, 2.2, 1.0),
        ),
        create_ambient_light_cmd(
            inner_realm_id,
            inner_light_id + 1,
            Vec4::new(0.32, 0.24, 0.20, 1.0),
            0.6,
        ),
        create_standard_material_cmd(
            left_material_id,
            "Demo7 Left Material",
            Vec4::new(0.10, 0.75, 0.95, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            inner_material_id,
            "Demo7 Inner Material",
            Vec4::new(0.98, 0.45, 0.12, 1.0),
            None,
            None,
        ),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
            crate::core::resources::CmdModelCreateArgs {
                realm_id: left_realm_id,
                model_id: left_model_id,
                label: Some("Demo7 Left Model".into()),
                geometry_id: left_geometry_id,
                material_id: Some(left_material_id),
                transform: Mat4::IDENTITY,
                layer_mask: 0xFFFF_FFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: false,
                outline_color: Vec4::ZERO,
            },
        )),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
            crate::core::resources::CmdModelCreateArgs {
                realm_id: inner_realm_id,
                model_id: inner_model_id,
                label: Some("Demo7 Inner Model".into()),
                geometry_id: inner_geometry_id,
                material_id: Some(inner_material_id),
                transform: Mat4::IDENTITY,
                layer_mask: 0xFFFF_FFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: false,
                outline_color: Vec4::ZERO,
            },
        )),
        EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
            document_id: doc_id,
            realm_id: ui_realm_id,
            rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
            theme_id: None,
        }),
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            document_id: doc_id,
            version: 1,
            ops: build_ui_tree(
                root_split_id,
                top_panel_id,
                bottom_panel_id,
                title_text_id,
                main_text_id,
                inner_text_id,
                viewport_id,
                inner_target_id,
            ),
        }),
        EngineCmd::CmdInputTargetListenerUpsert(CmdInputTargetListenerUpsertArgs {
            listener_id: main_listener_id,
            target_id: left_target_id,
            enabled: true,
            events: vec!["pointer-move".into(), "pointer-button".into()],
            sample_percent: 100,
        }),
        EngineCmd::CmdInputTargetListenerUpsert(CmdInputTargetListenerUpsertArgs {
            listener_id: inner_listener_id,
            target_id: inner_target_id,
            enabled: true,
            events: vec!["pointer-move".into(), "pointer-button".into()],
            sample_percent: 100,
        }),
    ];
    setup_cmds.extend(hud.setup_commands(ui_realm_id));
    let _ = send_commands(setup_cmds);
    let _ = receive_responses();

    let mut ui_version = 1_u64;
    let mut main_text = String::from("Main target: aguardando eventos de ponteiro...");
    let mut inner_text = String::from("Inner target: aguardando eventos de ponteiro...");
    let mut last_ui_push = std::time::Instant::now();

    run_loop_with_events(
        ctx.window_id,
        None,
        move |total_ms, delta_ms| {
            let mut cmds = hud.frame_commands(total_ms, delta_ms);
            cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Update(
                crate::core::resources::CmdModelUpdateArgs {
                    realm_id: left_realm_id,
                    model_id: left_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_rotation_y(total_ms as f32 * 0.0007)
                            * Mat4::from_rotation_x(total_ms as f32 * 0.0004),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                },
            )));
            cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Update(
                crate::core::resources::CmdModelUpdateArgs {
                    realm_id: inner_realm_id,
                    model_id: inner_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_rotation_y(-(total_ms as f32) * 0.0008)
                            * Mat4::from_rotation_z(total_ms as f32 * 0.0003),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                },
            )));
            cmds
        },
        move |event| {
            let EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id,
                event_type,
                position_global,
                position_target,
                ..
            }) = event
            else {
                return false;
            };

            let line = format!(
                "{} | global={} | target={}",
                event_type,
                format_vec2(position_global),
                format_vec2(position_target)
            );
            let text_slot = if listener_id == main_listener_id {
                &mut main_text
            } else if listener_id == inner_listener_id {
                &mut inner_text
            } else {
                return false;
            };
            if *text_slot == line {
                return false;
            }

            *text_slot = line;
            if last_ui_push.elapsed() < Duration::from_millis(60) {
                return false;
            }
            last_ui_push = std::time::Instant::now();
            ui_version = ui_version.saturating_add(1);
            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                document_id: doc_id,
                version: ui_version,
                ops: vec![
                    UiOp::Set {
                        node_id: main_text_id,
                        props: UiNodeProps::Text {
                            text: main_text.clone(),
                            size: Some(16.0),
                            color: Some(UiColor {
                                r: 24,
                                g: 34,
                                b: 52,
                                a: 255,
                            }),
                        },
                    },
                    UiOp::Set {
                        node_id: inner_text_id,
                        props: UiNodeProps::Text {
                            text: inner_text.clone(),
                            size: Some(16.0),
                            color: Some(UiColor {
                                r: 24,
                                g: 34,
                                b: 52,
                                a: 255,
                            }),
                        },
                    },
                ],
            })]);
            false
        },
    )
}

fn create_realm(kind: RealmKindDto) -> Option<u32> {
    let _ = send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind,
        importance: None,
        cache_policy: None,
        flags: None,
    })]);

    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
            if let CommandResponse::RealmCreate(result) = envelope.response
                && result.success
                && let Some(realm_id) = result.realm_id
            {
                return Some(realm_id);
            }
        }
        let _ = receive_events();
        std::thread::sleep(Duration::from_millis(2));
    }
    None
}
