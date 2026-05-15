use std::time::Duration;

use glam::{Mat4, Vec3, Vec4};

use super::pointer_listener_lab_ui::{build_ui_tree, format_vec2};
use super::{DemoContext, DemoIds, create_ui_realm};
use crate::demo::commands::{
    create_ambient_light_cmd, create_camera_cmd, create_pbr_material_cmd, create_point_light_cmd,
};
use crate::demo::hud::FpsHud;
use crate::demo::io::{receive_events, receive_responses, send_command, send_commands};
use crate::demo::loop_utils::run_loop_with_events;
use vulfram_core::core;
use vulfram_core::core::cmd::CmdEnvironmentUpsertArgs;
use vulfram_core::core::cmd::{CmdModelUpsertArgs, CommandResponse, EngineCmd, EngineEvent};
use vulfram_core::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use vulfram_core::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use vulfram_core::core::resources::{
    CmdEnvironmentUpdateArgs, EnvironmentConfig, SkyboxConfig, SkyboxMode,
};
use vulfram_core::core::system::SystemEvent;
use vulfram_core::core::target::{
    DimensionValue, TargetKind, TargetLayerLayout,
    cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs},
};
use vulfram_core::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs};
use vulfram_core::core::ui::types::{UiColor, UiNodeProps, UiOp};

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

    let window_target_id = ids.target_id + 7_000;
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
    let left_ground_geometry_id = ids.ground_geometry_id + 700;
    let inner_geometry_id = ids.geometry_id + 701;
    let inner_ground_geometry_id = ids.ground_geometry_id + 701;
    let left_material_id = ids.material_id + 700;
    let left_ground_material_id = ids.ground_material_id + 700;
    let inner_material_id = ids.material_id + 701;
    let inner_ground_material_id = ids.ground_material_id + 701;
    let left_env_id = ids.env_id + 700;
    let inner_env_id = ids.env_id + 701;
    let left_model_id = ids.model_id + 700;
    let left_ground_model_id = ids.ground_model_id + 700;
    let inner_model_id = ids.model_id + 701;
    let inner_ground_model_id = ids.ground_model_id + 701;
    let left_light_id = ids.light_id + 700;
    let inner_light_id = ids.light_id + 701;

    let main_listener_id = 970_001_u64;
    let inner_listener_id = 970_002_u64;

    let left_width = 50.0;
    let right_left = 50.0;
    let right_width = 50.0;

    let mut hud = FpsHud::new(7);
    let mut setup_cmds: Vec<EngineCmd> = Vec::new();

    setup_cmds.push(EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
        target_id: window_target_id,
        kind: TargetKind::Window,
        window_id: Some(ctx.window_id),
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    }));
    setup_cmds.push(EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
        window_id: ctx.window_id,
        config: ShadowConfig {
            normal_bias: 0.01,
            smoothing: 1,
            ..Default::default()
        },
    }));
    setup_cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id: left_realm_id,
        target_id: window_target_id,
        layout: TargetLayerLayout {
            left: DimensionValue::Percent(0.0),
            top: DimensionValue::Percent(0.0),
            width: DimensionValue::Percent(left_width),
            height: DimensionValue::Percent(100.0),
            enabled: true,
            opacity: 1.0,
            z_index: 0,
            blend_mode: 0,
            clip: None,
        },
        camera_id: Some(left_camera_id),
        environment_id: Some(left_env_id),
    }));
    setup_cmds.push(EngineCmd::CmdEnvironmentUpsert(
        CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
            environment_id: left_env_id,
            config: EnvironmentConfig {
                clear_color: Vec4::new(0.05, 0.09, 0.16, 1.0),
                skybox: SkyboxConfig {
                    mode: SkyboxMode::None,
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ));
    setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(
        vulfram_core::core::resources::CmdPrimitiveGeometryCreateArgs {
            geometry_id: left_geometry_id,
            label: Some("Demo7 Left Cube".into()),
            shape: vulfram_core::core::resources::PrimitiveShape::Cube,
            options: None,
        },
    ));
    setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(
        vulfram_core::core::resources::CmdPrimitiveGeometryCreateArgs {
            geometry_id: left_ground_geometry_id,
            label: Some("Demo7 Left Ground".into()),
            shape: vulfram_core::core::resources::PrimitiveShape::Plane,
            options: None,
        },
    ));
    setup_cmds.push(create_camera_cmd(
        left_realm_id,
        left_camera_id,
        "Demo7 Left Camera",
        Mat4::look_at_rh(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO, Vec3::Y).inverse(),
    ));
    setup_cmds.push(create_point_light_cmd(
        left_realm_id,
        left_light_id,
        Vec4::new(2.4, 3.2, 2.0, 1.0),
        20.0,
    ));
    setup_cmds.push(create_ambient_light_cmd(
        left_realm_id,
        left_light_id + 1,
        Vec4::new(0.22, 0.28, 0.38, 1.0),
        0.12,
    ));
    setup_cmds.push(create_pbr_material_cmd(
        left_material_id,
        "Demo7 Left Material",
        Vec4::new(0.10, 0.75, 0.95, 1.0),
        0.55,
        0.25,
    ));
    setup_cmds.push(create_pbr_material_cmd(
        left_ground_material_id,
        "Demo7 Left Ground Material",
        Vec4::new(0.16, 0.22, 0.30, 1.0),
        0.05,
        0.95,
    ));
    setup_cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
        vulfram_core::core::resources::CmdModelCreateArgs {
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
    )));
    setup_cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
        vulfram_core::core::resources::CmdModelCreateArgs {
            realm_id: left_realm_id,
            model_id: left_ground_model_id,
            label: Some("Demo7 Left Ground Model".into()),
            geometry_id: left_ground_geometry_id,
            material_id: Some(left_ground_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.2, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::splat(16.0)),
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        },
    )));

    setup_cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id: ui_realm_id,
        target_id: window_target_id,
        layout: TargetLayerLayout {
            left: DimensionValue::Percent(right_left),
            top: DimensionValue::Percent(0.0),
            width: DimensionValue::Percent(right_width),
            height: DimensionValue::Percent(100.0),
            enabled: true,
            opacity: 1.0,
            z_index: 2,
            blend_mode: 0,
            clip: None,
        },
        camera_id: None,
        environment_id: None,
    }));
    setup_cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: doc_id,
        realm_id: ui_realm_id,
        rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
        theme_id: None,
    }));
    setup_cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
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
    }));
    setup_cmds.extend(hud.setup_commands(ui_realm_id));

    setup_cmds.push(EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
        target_id: inner_target_id,
        kind: TargetKind::Texture,
        window_id: None,
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: Some(1),
    }));
    setup_cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id: inner_realm_id,
        target_id: inner_target_id,
        layout: TargetLayerLayout {
            left: DimensionValue::Percent(right_left),
            top: DimensionValue::Percent(45.0),
            width: DimensionValue::Percent(right_width),
            height: DimensionValue::Percent(55.0),
            enabled: true,
            opacity: 1.0,
            z_index: 3,
            blend_mode: 0,
            clip: None,
        },
        camera_id: Some(inner_camera_id),
        environment_id: Some(inner_env_id),
    }));
    setup_cmds.push(EngineCmd::CmdEnvironmentUpsert(
        CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
            environment_id: inner_env_id,
            config: EnvironmentConfig {
                clear_color: Vec4::new(0.10, 0.07, 0.12, 1.0),
                skybox: SkyboxConfig {
                    mode: SkyboxMode::None,
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ));
    setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(
        vulfram_core::core::resources::CmdPrimitiveGeometryCreateArgs {
            geometry_id: inner_geometry_id,
            label: Some("Demo7 Inner Cube".into()),
            shape: vulfram_core::core::resources::PrimitiveShape::Cube,
            options: None,
        },
    ));
    setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(
        vulfram_core::core::resources::CmdPrimitiveGeometryCreateArgs {
            geometry_id: inner_ground_geometry_id,
            label: Some("Demo7 Inner Ground".into()),
            shape: vulfram_core::core::resources::PrimitiveShape::Plane,
            options: None,
        },
    ));
    setup_cmds.push(create_camera_cmd(
        inner_realm_id,
        inner_camera_id,
        "Demo7 Inner Camera",
        Mat4::look_at_rh(Vec3::new(0.0, 2.2, 4.5), Vec3::ZERO, Vec3::Y).inverse(),
    ));
    setup_cmds.push(create_point_light_cmd(
        inner_realm_id,
        inner_light_id,
        Vec4::new(1.9, 3.0, 1.8, 1.0),
        20.0,
    ));
    setup_cmds.push(create_ambient_light_cmd(
        inner_realm_id,
        inner_light_id + 1,
        Vec4::new(0.32, 0.24, 0.20, 1.0),
        0.12,
    ));
    setup_cmds.push(create_pbr_material_cmd(
        inner_material_id,
        "Demo7 Inner Material",
        Vec4::new(0.98, 0.45, 0.12, 1.0),
        0.35,
        0.40,
    ));
    setup_cmds.push(create_pbr_material_cmd(
        inner_ground_material_id,
        "Demo7 Inner Ground Material",
        Vec4::new(0.24, 0.18, 0.16, 1.0),
        0.03,
        0.96,
    ));
    setup_cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
        vulfram_core::core::resources::CmdModelCreateArgs {
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
    )));
    setup_cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(
        vulfram_core::core::resources::CmdModelCreateArgs {
            realm_id: inner_realm_id,
            model_id: inner_ground_model_id,
            label: Some("Demo7 Inner Ground Model".into()),
            geometry_id: inner_ground_geometry_id,
            material_id: Some(inner_ground_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.2, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::splat(14.0)),
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        },
    )));

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
                vulfram_core::core::resources::CmdModelUpdateArgs {
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
                vulfram_core::core::resources::CmdModelUpdateArgs {
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
    let _ = receive_responses();
    let _ = receive_events();
    let command_id = send_command(EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind,
        importance: None,
        cache_policy: None,
        flags: None,
    }))?;

    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
            if envelope.id != command_id {
                continue;
            }
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
