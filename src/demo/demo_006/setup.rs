use std::time::Duration;

use glam::{Mat4, Vec2, Vec3, Vec4};

use super::maps::{build_bind_cmds, build_target_cmds, Demo006BindRealms};
use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentUpdateArgs, CmdModelCreateArgs,
    CmdPrimitiveGeometryCreateArgs, EnvironmentConfig, MsaaConfig, PostProcessConfig,
    PrimitiveShape, SkyboxConfig, SkyboxMode,
};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::types::{
    UiAnim, UiAnimEasing, UiAnimSpec, UiImageSource, UiLayout, UiLayoutDirection, UiLength, UiNode,
    UiNodeKind, UiNodeProps, UiOp, UiPadding, UiSize, UiThemeValue,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{
    DemoContext, create_ambient_light_cmd, create_floor_cmd, create_point_light_cmd,
    create_standard_material_cmd,
};

#[derive(Debug, Clone, Copy)]
pub struct Demo006Ids {
    pub geometry_cube_id: u32,
    pub geometry_plane_id: u32,
    pub material_main_id: u32,
    pub material_ui_id: u32,
    pub material_view_id: u32,
    pub material_panel_3d_id: u32,
    pub camera_main_id: u32,
    pub camera_view_id: u32,
    pub model_main_id: u32,
    pub model_ui_plane_id: u32,
    pub model_view_id: u32,
    pub model_panel_3d_id: u32,
    pub texture_ui_id: u32,
    pub texture_panel_3d_id: u32,
    pub ui_document_id: u32,
    pub ui_node_root: u32,
    pub ui_node_title: u32,
    pub ui_node_image: u32,
    pub ui_node_button: u32,
    pub ui_document_panel_3d_id: u32,
    pub ui_panel_3d_root: u32,
    pub ui_panel_3d_title: u32,
    pub ui_panel_3d_input: u32,
    pub ui_panel_3d_button: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo006RealmIds {
    pub _host_realm_main: u32,
    pub _realm_ui: u32,
    pub _realm_ui_panel_3d: u32,
    pub _realm_view: u32,
    pub target_texture_ui: u64,
    pub target_texture_view: u64,
    pub target_texture_panel_3d: u64,
}

pub struct Demo006Setup {
    pub ids: Demo006Ids,
    pub post_config: PostProcessConfig,
}

impl Demo006Setup {
    pub fn new() -> Self {
        let ids = Demo006Ids {
            geometry_cube_id: 1200,
            geometry_plane_id: 1201,
            material_main_id: 1210,
            material_ui_id: 1211,
            material_view_id: 1212,
            material_panel_3d_id: 1213,
            camera_main_id: 10,
            camera_view_id: 11,
            model_main_id: 1300,
            model_ui_plane_id: 1301,
            model_view_id: 1302,
            model_panel_3d_id: 1303,
            texture_ui_id: 1400,
            texture_panel_3d_id: 1401,
            ui_document_id: 1500,
            ui_node_root: 1501,
            ui_node_title: 1502,
            ui_node_image: 1503,
            ui_node_button: 1504,
            ui_document_panel_3d_id: 1510,
            ui_panel_3d_root: 1511,
            ui_panel_3d_title: 1512,
            ui_panel_3d_input: 1513,
            ui_panel_3d_button: 1514,
        };

        let post_config = build_demo_006_post_config();

        Self { ids, post_config }
    }

    pub fn apply(&self, ctx: DemoContext) -> Demo006RealmIds {
        let window_main = ctx.window_id;
        let host_realm_main = ctx.realm_id;

        let realm_ui = create_realm(RealmKindDto::TwoD, Some(window_main));
        let realm_ui_panel_3d = create_realm(RealmKindDto::TwoD, Some(window_main));
        let realm_view = create_realm(RealmKindDto::ThreeD, Some(window_main));

        let (target_ids, mut map_cmds) = build_target_cmds(window_main);
        let bind_cmds = build_bind_cmds(
            target_ids,
            Demo006BindRealms {
                host_main: host_realm_main,
                ui: realm_ui,
                ui_panel_3d: realm_ui_panel_3d,
                view: realm_view,
            },
        );
        map_cmds.extend(bind_cmds);

        assert_eq!(send_commands(map_cmds), VulframResult::Success);

        let setup_cmds = vec![
            EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
                window_id: window_main,
                config: EnvironmentConfig {
                    msaa: MsaaConfig {
                        enabled: true,
                        sample_count: 4,
                    },
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::Procedural,
                        intensity: 0.8,
                        rotation: 0.0,
                        ground_color: Vec3::new(0.05, 0.05, 0.08),
                        horizon_color: Vec3::new(0.2, 0.25, 0.35),
                        sky_color: Vec3::new(0.12, 0.22, 0.4),
                        cubemap_texture_id: None,
                    },
                    post: self.post_config.clone(),
                },
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_cube_id,
                label: Some("Demo 006 Cube".into()),
                shape: PrimitiveShape::Cube,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_plane_id,
                label: Some("Demo 006 Plane".into()),
                shape: PrimitiveShape::Plane,
                options: None,
            }),
            EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
                camera_id: self.ids.camera_main_id,
                label: Some("Demo 006 Camera".into()),
                transform: Mat4::look_at_rh(Vec3::new(0.0, 3.5, 9.0), Vec3::ZERO, Vec3::Y)
                    .inverse(),
                kind: CameraKind::Perspective,
                flags: 0,
                near_far: Vec2::new(0.1, 100.0),
                layer_mask: 0xFFFFFFFF,
                order: 0,
                view_position: None,
                ortho_scale: 10.0,
            }),
            EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
                camera_id: self.ids.camera_view_id,
                label: Some("Demo 006 View Camera".into()),
                transform: Mat4::look_at_rh(Vec3::new(0.0, 2.0, 5.5), Vec3::ZERO, Vec3::Y)
                    .inverse(),
                kind: CameraKind::Perspective,
                flags: 0,
                near_far: Vec2::new(0.1, 80.0),
                layer_mask: 0xFFFFFFFF,
                order: 1,
                view_position: None,
                ortho_scale: 10.0,
            }),
            create_point_light_cmd(window_main, 120, Vec4::new(3.0, 6.0, 2.0, 1.0)),
            create_ambient_light_cmd(window_main, 121, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.7),
            create_standard_material_cmd(
                window_main,
                self.ids.material_main_id,
                "Demo 006 Main",
                Vec4::new(0.2, 0.6, 0.9, 1.0),
                None,
                None,
            ),
            EngineCmd::CmdTextureBindTarget(crate::core::resources::CmdTextureBindTargetArgs {
                window_id: window_main,
                texture_id: self.ids.texture_ui_id,
                target_id: target_ids.texture_ui,
                label: Some("Demo 006 UI Texture".into()),
            }),
            EngineCmd::CmdTextureBindTarget(crate::core::resources::CmdTextureBindTargetArgs {
                window_id: window_main,
                texture_id: self.ids.texture_panel_3d_id,
                target_id: target_ids.texture_panel_3d,
                label: Some("Demo 006 Panel 3D Texture".into()),
            }),
            create_standard_material_cmd(
                window_main,
                self.ids.material_ui_id,
                "Demo 006 UI Material",
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Some(self.ids.texture_ui_id),
                None,
            ),
            create_standard_material_cmd(
                window_main,
                self.ids.material_panel_3d_id,
                "Demo 006 UIPlane Material",
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Some(self.ids.texture_panel_3d_id),
                Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ),
            create_standard_material_cmd(
                window_main,
                self.ids.material_view_id,
                "Demo 006 View Material",
                Vec4::new(0.9, 0.3, 0.25, 1.0),
                None,
                None,
            ),
            create_floor_cmd(window_main, self.ids.geometry_plane_id, self.ids.material_main_id),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id: window_main,
                model_id: self.ids.model_main_id,
                label: Some("Demo 006 Main Cube".into()),
                geometry_id: self.ids.geometry_cube_id,
                material_id: Some(self.ids.material_main_id),
                transform: Mat4::from_translation(Vec3::new(-2.2, 0.4, 0.0))
                    * Mat4::from_scale(Vec3::splat(1.2)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.1, 0.8, 0.9, 1.0),
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id: window_main,
                model_id: self.ids.model_ui_plane_id,
                label: Some("Demo 006 UI Plane".into()),
                geometry_id: self.ids.geometry_plane_id,
                material_id: Some(self.ids.material_ui_id),
                transform: Mat4::from_translation(Vec3::new(2.6, 0.8, -1.2))
                    * Mat4::from_scale(Vec3::new(2.0, 1.0, 1.3)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: false,
                receive_shadow: false,
                cast_outline: false,
                outline_color: Vec4::ZERO,
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id: window_main,
                model_id: self.ids.model_view_id,
                label: Some("Demo 006 View Cube".into()),
                geometry_id: self.ids.geometry_cube_id,
                material_id: Some(self.ids.material_view_id),
                transform: Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0))
                    * Mat4::from_scale(Vec3::splat(0.9)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.9, 0.4, 0.2, 1.0),
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id: window_main,
                model_id: self.ids.model_panel_3d_id,
                label: Some("Demo 006 UIPlane".into()),
                geometry_id: self.ids.geometry_plane_id,
                material_id: Some(self.ids.material_panel_3d_id),
                transform: Mat4::from_translation(Vec3::new(0.0, 1.4, 0.0))
                    * Mat4::from_rotation_y(-std::f32::consts::FRAC_PI_2)
                    * Mat4::from_scale(Vec3::new(2.2, 1.4, 1.0)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: false,
                receive_shadow: false,
                cast_outline: true,
                outline_color: Vec4::new(0.2, 0.9, 0.7, 1.0),
            }),
        ];

        assert_eq!(send_commands(setup_cmds), VulframResult::Success);
        let _ = receive_responses();

        let ui_cmds = build_ui_cmds(self.ids, realm_ui, target_ids.texture_view);
        let ui_panel_3d_cmds = build_ui_cmds_panel_3d(self.ids, realm_ui_panel_3d);
        assert_eq!(
            send_commands(ui_cmds.into_iter().chain(ui_panel_3d_cmds).collect()),
            VulframResult::Success
        );
        let _ = receive_responses();

        Demo006RealmIds {
            _host_realm_main: host_realm_main,
            _realm_ui: realm_ui,
            _realm_ui_panel_3d: realm_ui_panel_3d,
            _realm_view: realm_view,
            target_texture_ui: target_ids.texture_ui,
            target_texture_view: target_ids.texture_view,
            target_texture_panel_3d: target_ids.texture_panel_3d,
        }
    }
}

fn build_ui_cmds(ids: Demo006Ids, realm_ui: u32, target_view: u64) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    cmds.push(EngineCmd::CmdUiThemeDefine(CmdUiThemeDefineArgs {
        theme_id: 1,
        version: None,
        data: std::collections::HashMap::from([
            ("fontSize".into(), UiThemeValue::Float(16.0)),
            ("textColor".into(), UiThemeValue::String("#E8F2FF".into())),
            ("panelFill".into(), UiThemeValue::String("#0E1B2BCC".into())),
            ("accentColor".into(), UiThemeValue::String("#5AD1FF".into())),
        ]),
    }));

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_document_id,
        realm_id: realm_ui,
        rect: glam::Vec4::new(0.0, 0.0, 520.0, 320.0),
        theme_id: Some(1),
    }));

    let root = UiNode {
        id: ids.ui_node_root,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 10.0,
                ..Default::default()
            },
            padding: Some(UiPadding {
                left: 12.0,
                top: 12.0,
                right: 12.0,
                bottom: 12.0,
            }),
            size: None,
            scroll_x: false,
            scroll_y: false,
        },
        anim: None,
        display: None,
        visible: None,
        opacity: Some(0.95),
        z_index: None,
    };

    let title = UiNode {
        id: ids.ui_node_title,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "Demo 006: UI + 3D".into(),
            size: Some(18.0),
            color: None,
        },
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let image = UiNode {
        id: ids.ui_node_image,
        kind: UiNodeKind::Image,
        props: UiNodeProps::Image {
            source: UiImageSource::Target(target_view),
            size: Some(UiSize {
                width: UiLength::Px(360.0),
                height: UiLength::Px(200.0),
            }),
        },
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let button = UiNode {
        id: ids.ui_node_button,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Trigger".into(),
            enabled: Some(true),
        },
        anim: Some(UiAnim {
            opacity: Some(UiAnimSpec {
                from: 0.4,
                to: 1.0,
                duration_ms: 600,
                easing: UiAnimEasing::EaseInOut,
            }),
            translate_y: Some(UiAnimSpec {
                from: 6.0,
                to: 0.0,
                duration_ms: 600,
                easing: UiAnimEasing::EaseInOut,
            }),
        }),
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: ids.ui_document_id,
        version: 1,
        ops: vec![
            UiOp::Add {
                parent: None,
                node: root,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_node_root),
                node: title,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_node_root),
                node: image,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_node_root),
                node: button,
                index: None,
            },
        ],
    }));

    cmds
}

fn build_ui_cmds_panel_3d(ids: Demo006Ids, realm_ui: u32) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_document_panel_3d_id,
        realm_id: realm_ui,
        rect: glam::Vec4::new(0.0, 0.0, 360.0, 240.0),
        theme_id: Some(1),
    }));

    let root = UiNode {
        id: ids.ui_panel_3d_root,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 8.0,
                ..Default::default()
            },
            padding: Some(UiPadding {
                left: 10.0,
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
            }),
            size: None,
            scroll_x: false,
            scroll_y: false,
        },
        anim: None,
        display: None,
        visible: None,
        opacity: Some(0.9),
        z_index: None,
    };

    let title = UiNode {
        id: ids.ui_panel_3d_title,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "3D Panel".into(),
            size: Some(16.0),
            color: None,
        },
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let input = UiNode {
        id: ids.ui_panel_3d_input,
        kind: UiNodeKind::Input,
        props: UiNodeProps::Input {
            value: "Digite aqui".into(),
            placeholder: Some("Texto...".into()),
            enabled: Some(true),
        },
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let button = UiNode {
        id: ids.ui_panel_3d_button,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Enviar".into(),
            enabled: Some(true),
        },
        anim: Some(UiAnim {
            opacity: Some(UiAnimSpec {
                from: 0.2,
                to: 1.0,
                duration_ms: 700,
                easing: UiAnimEasing::EaseInOut,
            }),
            translate_y: Some(UiAnimSpec {
                from: 8.0,
                to: 0.0,
                duration_ms: 700,
                easing: UiAnimEasing::EaseInOut,
            }),
        }),
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: ids.ui_document_panel_3d_id,
        version: 1,
        ops: vec![
            UiOp::Add {
                parent: None,
                node: root,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_3d_root),
                node: title,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_3d_root),
                node: input,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_3d_root),
                node: button,
                index: None,
            },
        ],
    }));

    cmds
}

fn build_demo_006_post_config() -> PostProcessConfig {
    PostProcessConfig {
        filter_enabled: true,
        filter_exposure: 1.0,
        filter_gamma: 2.2,
        filter_saturation: 1.0,
        filter_contrast: 1.0,
        filter_vignette: 0.4,
        filter_grain: 0.04,
        filter_chromatic_aberration: 0.0,
        filter_blur: 0.0,
        filter_sharpen: 0.2,
        filter_tonemap_mode: 1,
        outline_enabled: true,
        outline_strength: 0.6,
        outline_threshold: 0.2,
        outline_width: 2.0,
        outline_quality: 1.0,
        filter_posterize_steps: 0.0,
        cell_shading: false,
        ssao_enabled: true,
        ssao_strength: 0.7,
        ssao_radius: 0.9,
        ssao_bias: 0.02,
        ssao_power: 1.2,
        ssao_blur_radius: 2.0,
        ssao_blur_depth_threshold: 0.02,
        bloom_enabled: true,
        bloom_threshold: 1.0,
        bloom_knee: 0.8,
        bloom_intensity: 1.0,
        bloom_scatter: 1.0,
    }
}

fn create_realm(kind: RealmKindDto, host_window_id: Option<u32>) -> u32 {
    assert_eq!(
        send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind,
            output_surface_id: None,
            host_window_id,
            importance: Some(1),
            cache_policy: Some(0),
            flags: Some(0),
        })]),
        VulframResult::Success
    );
    wait_for_response(|response| match response {
        CommandResponse::RealmCreate(result) if result.success => result.realm_id,
        _ => None,
    })
    .expect("realm creation failed")
}

fn wait_for_response<F, T>(mut pick: F) -> Option<T>
where
    F: FnMut(CommandResponse) -> Option<T>,
{
    for _ in 0..120 {
        let responses = receive_responses();
        for response in responses {
            if let Some(value) = pick(response.response) {
                return Some(value);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(0, 0), VulframResult::Success);
    }
    None
}
