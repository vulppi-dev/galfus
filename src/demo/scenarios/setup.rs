use super::*;
use crate::core::resources::*;
mod setup_cases_tail;
pub(super) fn ui_button_op(parent: u32, node_id: u32, label: &str) -> UiOp {
    UiOp::Add {
        parent: Some(parent),
        node: UiNode {
            id: node_id,
            kind: UiNodeKind::Button,
            props: UiNodeProps::Button {
                label: label.into(),
                enabled: Some(true),
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index: Some(101),
        },
        index: None,
    }
}

pub(super) fn window_measurement_cmd(window_id: u32) -> EngineCmd {
    EngineCmd::CmdWindowMeasurement(CmdWindowMeasurementArgs {
        window_id,
        get_position: true,
        get_size: true,
        get_outer_size: true,
        get_surface_size: true,
        ..Default::default()
    })
}

pub(super) fn window_state_cmd(window_id: u32, state: EngineWindowState) -> EngineCmd {
    EngineCmd::CmdWindowState(CmdWindowStateArgs {
        window_id,
        state: Some(state),
        get_state: true,
        get_decorations: true,
        get_resizable: true,
        ..Default::default()
    })
}

pub(super) fn window_cursor_cmd(window_id: u32, icon: CursorIcon) -> EngineCmd {
    EngineCmd::CmdWindowCursor(CmdWindowCursorArgs {
        window_id,
        visible: Some(true),
        mode: None,
        icon: Some(icon),
    })
}

pub(super) fn base_scene_commands(ctx: DemoContext, ids: DemoIds) -> Vec<EngineCmd> {
    let cmds = vec![
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: ids.env_id,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::None,
                        ..Default::default()
                    },
                    post: PostProcessConfig {
                        filter_enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id: ids.target_id,
            kind: TargetKind::Window,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: ids.geometry_id,
            label: Some("Demo Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: ids.ground_geometry_id,
            label: Some("Demo Ground Plane".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        create_camera_cmd(
            ctx.realm_id,
            ids.camera_id,
            "Main Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 3.5, 9.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(
            ctx.realm_id,
            ids.light_id,
            Vec4::new(4.0, 6.0, 4.0, 1.0),
            8.0,
        ),
        create_ambient_light_cmd(
            ctx.realm_id,
            ids.light_id + 1,
            Vec4::new(0.2, 0.2, 0.2, 1.0),
            0.08,
        ),
        create_standard_material_cmd(
            ids.material_id,
            "Cube Material",
            Vec4::new(0.9, 0.5, 0.2, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            ids.ground_material_id,
            "Ground Material",
            Vec4::new(0.22, 0.24, 0.28, 1.0),
            None,
            None,
        ),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id: ctx.realm_id,
            model_id: ids.model_id,
            label: Some("Demo Cube Model".into()),
            geometry_id: ids.geometry_id,
            material_id: Some(ids.material_id),
            transform: Mat4::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id: ctx.realm_id,
            model_id: ids.ground_model_id,
            label: Some("Demo Ground Model".into()),
            geometry_id: ids.ground_geometry_id,
            material_id: Some(ids.ground_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.2, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::splat(18.0)),
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: ctx.realm_id,
            target_id: ids.target_id,
            layout: TargetLayerLayout::default(),
            camera_id: Some(ids.camera_id),
            environment_id: Some(ids.env_id),
        }),
        create_shadow_config_cmd(ctx.window_id),
    ];

    cmds
}

pub(super) fn aux_window_commands(window_id: u32, realm_id: u32, ids: DemoIds) -> Vec<EngineCmd> {
    let target_id = ids.target_id + 500;
    vec![
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id,
            kind: TargetKind::Window,
            window_id: Some(window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }),
        EngineCmd::CmdCameraUpsert(CmdCameraUpsertArgs::Create(CmdCameraCreateArgs {
            realm_id,
            camera_id: ids.camera_id + 500,
            label: Some("Aux Camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 3.0, 6.0), Vec3::ZERO, Vec3::Y).inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFF_FFFF,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        })),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id,
            target_id,
            layout: TargetLayerLayout::default(),
            camera_id: Some(ids.camera_id + 500),
            environment_id: None,
        }),
    ]
}

mod setup_cases;
pub(super) use setup_cases::extra_setup_commands;

pub(super) fn list_commands(window_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdModelList(CmdModelListArgs { window_id }),
        EngineCmd::CmdMaterialList(CmdMaterialListArgs { window_id }),
        EngineCmd::CmdTextureList(CmdTextureListArgs { window_id }),
        EngineCmd::CmdGeometryList(CmdGeometryListArgs { window_id }),
        EngineCmd::CmdLightList(CmdLightListArgs { window_id }),
        EngineCmd::CmdCameraList(CmdCameraListArgs { window_id }),
    ]
}
