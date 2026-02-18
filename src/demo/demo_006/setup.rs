use std::time::Duration;

use glam::{Mat4, Vec2, Vec3, Vec4};

use super::maps::{Demo006LayerRealms, build_layer_cmds, build_target_cmds};
use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentUpdateArgs, CmdMaterialCreateArgs,
    CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs, CmdTextureBindTargetArgs,
    EnvironmentConfig, MaterialKind, MaterialOptions, MaterialSampler, MsaaConfig,
    PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode, StandardOptions, SurfaceType,
};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiDocumentCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::types::{
    UiLayout, UiLayoutDirection, UiLength, UiNode, UiNodeKind, UiNodeProps, UiOp, UiPadding,
    UiSize, UiThemeValue,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{
    DemoContext, create_ambient_light_cmd, create_point_light_cmd, create_standard_material_cmd,
};

#[derive(Debug, Clone, Copy)]
pub struct Demo006Ids {
    pub camera_main_id: u32,
    pub light_ambient_id: u32,
    pub light_key_id: u32,
    pub geometry_cube_id: u32,
    pub material_cube_id: u32,
    pub model_cube_id: u32,
    pub geometry_realm_plane_id: u32,
    pub texture_ui_panel_id: u32,
    pub material_realm_plane_id: u32,
    pub model_realm_plane_id: u32,
    pub ui_document_id: u32,
    pub ui_root_id: u32,
    pub ui_title_id: u32,
    pub ui_body_id: u32,
    pub ui_input_id: u32,
    pub ui_button_add_id: u32,
    pub ui_button_remove_id: u32,
    pub ui_toggle_checkbox_id: u32,
    pub ui_viewport_document_id: u32,
    pub ui_viewport_root_id: u32,
    pub ui_viewport_node_id: u32,
    pub ui_panel_document_id: u32,
    pub ui_panel_root_id: u32,
    pub ui_panel_title_id: u32,
    pub ui_panel_body_id: u32,
    pub ui_panel_input_id: u32,
    pub ui_panel_button_add_id: u32,
    pub ui_panel_button_remove_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo006RealmIds {
    pub _realm_ui: u32,
    pub _realm_3d_embed: u32,
    pub _realm_ui_panel_3d: u32,
}

pub struct Demo006Setup {
    pub ids: Demo006Ids,
    pub post_config: PostProcessConfig,
}

impl Demo006Setup {
    pub fn new() -> Self {
        let ids = Demo006Ids {
            camera_main_id: 1,
            light_ambient_id: 11,
            light_key_id: 12,
            geometry_cube_id: 1200,
            material_cube_id: 1210,
            model_cube_id: 1300,
            geometry_realm_plane_id: 1220,
            texture_ui_panel_id: 1221,
            material_realm_plane_id: 1222,
            model_realm_plane_id: 1320,
            ui_document_id: 1500,
            ui_root_id: 1501,
            ui_title_id: 1502,
            ui_body_id: 1503,
            ui_input_id: 1504,
            ui_button_add_id: 1505,
            ui_button_remove_id: 1506,
            ui_toggle_checkbox_id: 1507,
            ui_viewport_document_id: 1508,
            ui_viewport_root_id: 1509,
            ui_viewport_node_id: 1510,
            ui_panel_document_id: 1600,
            ui_panel_root_id: 1601,
            ui_panel_title_id: 1602,
            ui_panel_body_id: 1603,
            ui_panel_input_id: 1604,
            ui_panel_button_add_id: 1605,
            ui_panel_button_remove_id: 1606,
        };

        let post_config = build_demo_006_post_config();

        Self { ids, post_config }
    }

    pub fn apply(&self, ctx: DemoContext) -> Demo006RealmIds {
        let window_main = ctx.window_id;
        let host_realm_main = ctx.realm_id;

        let realm_ui = create_realm(RealmKindDto::TwoD, Some(window_main));
        let realm_3d_embed = create_realm(RealmKindDto::ThreeD, Some(window_main));
        let realm_ui_panel_3d = create_realm(RealmKindDto::TwoD, Some(window_main));

        let (target_ids, mut map_cmds) = build_target_cmds(window_main);
        let layer_cmds = build_layer_cmds(
            target_ids,
            Demo006LayerRealms {
                host_main: host_realm_main,
                ui: realm_ui,
                realm_3d_embed: realm_3d_embed,
                ui_panel_3d: realm_ui_panel_3d,
            },
        );
        map_cmds.extend(layer_cmds);
        assert_eq!(send_commands(map_cmds), VulframResult::Success);

        let mut setup_cmds = vec![
            EngineCmd::CmdEnvironmentUpsert(crate::core::cmd::CmdEnvironmentUpsertArgs::Update(
                CmdEnvironmentUpdateArgs {
                    environment_id: window_main,
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
                        clear_color: Vec3::new(0.0, 0.0, 0.0),
                        post: self.post_config.clone(),
                    },
                },
            )),
            EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
                CmdCameraCreateArgs {
                    camera_id: self.ids.camera_main_id,
                    label: Some("Demo 006 Camera".into()),
                    transform: Mat4::look_at_rh(Vec3::new(0.0, 3.2, 8.5), Vec3::ZERO, Vec3::Y)
                        .inverse(),
                    kind: CameraKind::Perspective,
                    flags: 0,
                    near_far: Vec2::new(0.1, 80.0),
                    layer_mask: 0xFFFFFFFF,
                    order: 0,
                    view_position: None,
                    ortho_scale: 10.0,
                },
            )),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_cube_id,
                label: Some("Demo 006 Cube".into()),
                shape: PrimitiveShape::Cube,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_realm_plane_id,
                label: Some("Demo 006 RealmPlane".into()),
                shape: PrimitiveShape::Plane,
                options: None,
            }),
            create_standard_material_cmd(
                window_main,
                self.ids.material_cube_id,
                "Demo 006 Cube Material",
                Vec4::new(0.2, 0.8, 1.0, 1.0),
                None,
                None,
            ),
            create_ambient_light_cmd(
                window_main,
                self.ids.light_ambient_id,
                Vec4::new(0.85, 0.9, 1.0, 1.0),
                1.25,
            ),
            create_point_light_cmd(
                window_main,
                self.ids.light_key_id,
                Vec4::new(3.5, 4.5, 5.5, 1.0),
            ),
            EngineCmd::CmdTextureBindTarget(CmdTextureBindTargetArgs {
                window_id: window_main,
                texture_id: self.ids.texture_ui_panel_id,
                target_id: target_ids.texture_ui_panel_3d,
                label: Some("Demo 006 UI Panel Texture".into()),
            }),
            EngineCmd::CmdMaterialUpsert(crate::core::cmd::CmdMaterialUpsertArgs::Create(
                CmdMaterialCreateArgs {
                    window_id: window_main,
                    material_id: self.ids.material_realm_plane_id,
                    label: Some("Demo 006 RealmPlane Material".into()),
                    kind: MaterialKind::Standard,
                    options: Some(MaterialOptions::Standard(StandardOptions {
                        base_color: Vec4::ONE,
                        surface_type: SurfaceType::Transparent,
                        base_tex_id: Some(self.ids.texture_ui_panel_id),
                        base_sampler: Some(MaterialSampler::LinearClamp),
                        ..Default::default()
                    })),
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id: window_main,
                    model_id: self.ids.model_realm_plane_id,
                    label: Some("Demo 006 RealmPlane Model".into()),
                    geometry_id: self.ids.geometry_realm_plane_id,
                    material_id: Some(self.ids.material_realm_plane_id),
                    transform: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.8))
                        * Mat4::from_rotation_y(std::f32::consts::PI - 0.35)
                        * Mat4::from_rotation_x(-0.08)
                        * Mat4::from_scale(Vec3::new(2.4, 1.0, 1.0)),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: false,
                    receive_shadow: false,
                    cast_outline: false,
                    outline_color: Vec4::ZERO,
                },
            )),
        ];

        let cube_positions = [
            Vec3::new(0.0, 0.8, 0.0),
            Vec3::new(-2.4, 0.8, 0.2),
            Vec3::new(2.4, 0.8, -0.2),
            Vec3::new(-1.2, 2.0, -0.8),
            Vec3::new(1.2, 2.0, 0.8),
            Vec3::new(0.0, -0.6, -1.2),
            Vec3::new(-3.2, 0.2, -2.0),
            Vec3::new(3.2, 0.2, -2.0),
            Vec3::new(-1.9, 1.3, 1.6),
            Vec3::new(1.9, 1.3, 1.6),
        ];

        for (index, position) in cube_positions.iter().enumerate() {
            setup_cmds.push(EngineCmd::CmdModelUpsert(
                crate::core::cmd::CmdModelUpsertArgs::Create(CmdModelCreateArgs {
                    window_id: window_main,
                    model_id: self.ids.model_cube_id + index as u32,
                    label: Some(format!("Demo 006 Cube {}", index + 1)),
                    geometry_id: self.ids.geometry_cube_id,
                    material_id: Some(self.ids.material_cube_id),
                    transform: Mat4::from_translation(*position)
                        * Mat4::from_scale(Vec3::splat(1.4)),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: Vec4::new(0.1, 0.9, 1.0, 1.0),
                }),
            ));
        }

        assert_eq!(send_commands(setup_cmds), VulframResult::Success);
        assert_command_batch_success(20, "setup");

        let mut ui_cmds = build_ui_cmds(self.ids, realm_ui, target_ids.widget_realm_viewport);
        ui_cmds.extend(build_ui_panel_3d_cmds(self.ids, realm_ui_panel_3d));
        assert_eq!(send_commands(ui_cmds), VulframResult::Success);
        assert_command_batch_success(7, "ui");

        Demo006RealmIds {
            _realm_ui: realm_ui,
            _realm_3d_embed: realm_3d_embed,
            _realm_ui_panel_3d: realm_ui_panel_3d,
        }
    }
}

fn build_ui_panel_3d_cmds(ids: Demo006Ids, realm_ui_panel_3d: u32) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_panel_document_id,
        realm_id: realm_ui_panel_3d,
        rect: glam::Vec4::new(0.0, 0.0, 280.0, 180.0),
        theme_id: Some(1),
    }));

    let root = UiNode {
        id: ids.ui_panel_root_id,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 8.0,
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
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: Some(1.0),
        z_index: None,
    };

    let title = UiNode {
        id: ids.ui_panel_title_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "UIPanel no lado 3D".into(),
            size: Some(16.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let body = UiNode {
        id: ids.ui_panel_body_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "Contador painel: 0".into(),
            size: Some(14.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };
    let input = UiNode {
        id: ids.ui_panel_input_id,
        kind: UiNodeKind::Input,
        props: UiNodeProps::Input {
            value: "Digite no painel 3D".into(),
            placeholder: Some("Texto no UIPanel".into()),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };
    let button_add = UiNode {
        id: ids.ui_panel_button_add_id,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Painel +".into(),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };
    let button_remove = UiNode {
        id: ids.ui_panel_button_remove_id,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Painel -".into(),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: ids.ui_panel_document_id,
        version: 1,
        ops: vec![
            UiOp::Add {
                parent: None,
                node: root,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_root_id),
                node: title,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_root_id),
                node: body,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_root_id),
                node: input,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_root_id),
                node: button_add,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_panel_root_id),
                node: button_remove,
                index: None,
            },
        ],
    }));

    cmds
}

fn assert_command_batch_success(expected_responses: usize, tag: &str) {
    let mut received = 0usize;
    for _ in 0..120 {
        let responses = receive_responses();
        received += responses.len();
        for response in responses {
            match response.response {
                CommandResponse::EnvironmentUpsert(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::CameraUpsert(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::PrimitiveGeometryCreate(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::MaterialUpsert(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::LightUpsert(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::ModelUpsert(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::UiThemeDefine(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::UiDocumentCreate(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                CommandResponse::UiApplyOps(result) => {
                    assert!(result.success, "[demo006:{tag}] {}", result.message);
                }
                _ => {}
            }
        }
        if received >= expected_responses {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(0, 0), VulframResult::Success);
    }
    assert!(
        received >= expected_responses,
        "[demo006:{tag}] incomplete responses: got {received}, expected at least {expected_responses}"
    );
}

fn build_ui_cmds(
    ids: Demo006Ids,
    realm_ui: u32,
    widget_realm_viewport_target: u64,
) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    cmds.push(EngineCmd::CmdUiThemeDefine(CmdUiThemeDefineArgs {
        theme_id: 1,
        version: None,
        data: std::collections::HashMap::from([
            ("fontSize".into(), UiThemeValue::Float(16.0)),
            ("textColor".into(), UiThemeValue::String("#E6E6E6".into())),
            ("panelFill".into(), UiThemeValue::String("#1B1D21".into())),
            ("accentColor".into(), UiThemeValue::String("#5AD1FF".into())),
        ]),
        font_data: std::collections::HashMap::new(),
        font_families: std::collections::HashMap::new(),
    }));

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_document_id,
        realm_id: realm_ui,
        rect: glam::Vec4::new(0.0, 0.0, 360.0, 720.0),
        theme_id: Some(1),
    }));

    cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
        document_id: ids.ui_viewport_document_id,
        realm_id: realm_ui,
        rect: glam::Vec4::new(20.0, 430.0, 320.0, 240.0),
        theme_id: Some(1),
    }));

    let root = UiNode {
        id: ids.ui_root_id,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout {
                direction: UiLayoutDirection::Column,
                gap: 12.0,
                ..Default::default()
            },
            padding: Some(UiPadding {
                left: 18.0,
                top: 18.0,
                right: 18.0,
                bottom: 18.0,
            }),
            size: None,
            scroll_x: false,
            scroll_y: false,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: Some(1.0),
        z_index: None,
    };

    let title = UiNode {
        id: ids.ui_title_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "Demo 006: UI + 3D".into(),
            size: Some(20.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let body = UiNode {
        id: ids.ui_body_id,
        kind: UiNodeKind::Text,
        props: UiNodeProps::Text {
            text: "Lado esquerdo UI iterativa, lado direito com cubo girando.".into(),
            size: Some(15.0),
            color: None,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let input = UiNode {
        id: ids.ui_input_id,
        kind: UiNodeKind::Input,
        props: UiNodeProps::Input {
            value: "Texto inicial".into(),
            placeholder: Some("Digite algo...".into()),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let button_add = UiNode {
        id: ids.ui_button_add_id,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Adicionar".into(),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    let button_remove = UiNode {
        id: ids.ui_button_remove_id,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "Remover".into(),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    // Checkbox simulated with a toggle button because UiNodeKind currently has no native checkbox.
    let toggle_checkbox = UiNode {
        id: ids.ui_toggle_checkbox_id,
        kind: UiNodeKind::Button,
        props: UiNodeProps::Button {
            label: "[ ] Habilitar efeito".into(),
            enabled: Some(true),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
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
                parent: Some(ids.ui_root_id),
                node: title,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_root_id),
                node: body,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_root_id),
                node: input,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_root_id),
                node: button_add,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_root_id),
                node: button_remove,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_root_id),
                node: toggle_checkbox,
                index: None,
            },
        ],
    }));

    let viewport_root = UiNode {
        id: ids.ui_viewport_root_id,
        kind: UiNodeKind::Container,
        props: UiNodeProps::Container {
            layout: UiLayout::default(),
            padding: None,
            size: Some(UiSize {
                width: UiLength::Fill,
                height: UiLength::Fill,
            }),
            scroll_x: false,
            scroll_y: false,
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: Some(1.0),
        z_index: None,
    };

    let viewport_node = UiNode {
        id: ids.ui_viewport_node_id,
        kind: UiNodeKind::WidgetRealmViewport,
        props: UiNodeProps::WidgetRealmViewport {
            target_id: widget_realm_viewport_target,
            size: Some(UiSize {
                width: UiLength::Fill,
                height: UiLength::Fill,
            }),
        },
        tooltip: None,
        context_menu: None,
        anim: None,
        display: None,
        visible: None,
        opacity: None,
        z_index: None,
    };

    cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
        document_id: ids.ui_viewport_document_id,
        version: 1,
        ops: vec![
            UiOp::Add {
                parent: None,
                node: viewport_root,
                index: None,
            },
            UiOp::Add {
                parent: Some(ids.ui_viewport_root_id),
                node: viewport_node,
                index: None,
            },
        ],
    }));

    cmds
}

fn build_demo_006_post_config() -> PostProcessConfig {
    PostProcessConfig {
        filter_enabled: false,
        filter_exposure: 1.0,
        filter_gamma: 2.2,
        filter_saturation: 1.0,
        filter_contrast: 1.0,
        filter_vignette: 0.2,
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
        bloom_intensity: 0.9,
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
