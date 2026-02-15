use std::time::Duration;

use glam::{Mat4, Vec2, Vec3, Vec4};

use super::maps::{Demo007LayerRealms, build_layer_cmds, build_target_cmds};
use super::ui::build_ui_cmds;
use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentUpdateArgs, CmdMaterialCreateArgs,
    CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs, EnvironmentConfig, MaterialKind,
    MaterialOptions, MsaaConfig, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode,
    StandardOptions,
};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{DemoContext, create_ambient_light_cmd, create_point_light_cmd};

#[derive(Debug, Clone, Copy)]
pub struct Demo007Ids {
    pub camera_a_id: u32,
    pub camera_b_id: u32,
    pub camera_c_id: u32,
    pub camera_d_id: u32,
    pub geometry_cube_id: u32,
    pub geometry_cone_id: u32,
    pub geometry_torus_id: u32,
    pub material_shape_id: u32,
    pub model_ids: [u32; 3],
    pub ui_document_id: u32,
    pub ui_root_id: u32,
    pub ui_title_id: u32,
    pub ui_grid_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo007RealmIds {
    pub _realm_ui: u32,
    pub _realm_3d: u32,
}

pub struct Demo007Setup {
    pub ids: Demo007Ids,
}

impl Demo007Setup {
    pub fn new() -> Self {
        Self {
            ids: Demo007Ids {
                camera_a_id: 7101,
                camera_b_id: 7102,
                camera_c_id: 7103,
                camera_d_id: 7104,
                geometry_cube_id: 7201,
                geometry_cone_id: 7202,
                geometry_torus_id: 7203,
                material_shape_id: 7210,
                model_ids: [7301, 7302, 7303],
                ui_document_id: 7401,
                ui_root_id: 7402,
                ui_title_id: 7403,
                ui_grid_id: 7404,
            },
        }
    }

    pub fn apply(&self, ctx: DemoContext) -> Demo007RealmIds {
        let window_main = ctx.window_id;
        let host_realm_main = ctx.realm_id;

        let realm_ui = create_realm(RealmKindDto::TwoD, Some(window_main));
        let realm_3d = create_realm(RealmKindDto::ThreeD, Some(window_main));

        let (target_ids, mut map_cmds) = build_target_cmds(window_main);
        let layer_cmds = build_layer_cmds(
            target_ids,
            Demo007LayerRealms {
                host_main: host_realm_main,
                ui: realm_ui,
                realm_3d,
            },
        );
        map_cmds.extend(layer_cmds);
        assert_eq!(send_commands(map_cmds), VulframResult::Success);

        let post_config = PostProcessConfig {
            filter_enabled: false,
            filter_exposure: 1.0,
            filter_gamma: 2.2,
            filter_saturation: 1.0,
            filter_contrast: 1.0,
            filter_vignette: 0.15,
            filter_grain: 0.0,
            filter_chromatic_aberration: 0.0,
            filter_blur: 0.0,
            filter_sharpen: 0.2,
            filter_tonemap_mode: 1,
            outline_enabled: false,
            outline_strength: 0.0,
            outline_threshold: 0.2,
            outline_width: 1.0,
            outline_quality: 1.0,
            filter_posterize_steps: 0.0,
            cell_shading: false,
            ssao_enabled: false,
            ssao_strength: 0.0,
            ssao_radius: 1.0,
            ssao_bias: 0.02,
            ssao_power: 1.0,
            ssao_blur_radius: 0.0,
            ssao_blur_depth_threshold: 0.02,
            bloom_enabled: false,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_intensity: 0.0,
            bloom_scatter: 1.0,
        };

        let mut cmds = vec![
            EngineCmd::CmdEnvironmentUpsert(crate::core::cmd::CmdEnvironmentUpsertArgs::Update(
                CmdEnvironmentUpdateArgs {
                    window_id: window_main,
                    config: EnvironmentConfig {
                        msaa: MsaaConfig {
                            enabled: true,
                            sample_count: 4,
                        },
                        skybox: SkyboxConfig {
                            mode: SkyboxMode::Procedural,
                            intensity: 0.9,
                            rotation: 0.0,
                            ground_color: Vec3::new(0.04, 0.05, 0.08),
                            horizon_color: Vec3::new(0.18, 0.23, 0.34),
                            sky_color: Vec3::new(0.14, 0.24, 0.42),
                            cubemap_texture_id: None,
                        },
                        post: post_config,
                    },
                },
            )),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_cube_id,
                label: Some("Demo 007 Cube".into()),
                shape: PrimitiveShape::Cube,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_cone_id,
                label: Some("Demo 007 Cone (Pyramid)".into()),
                shape: PrimitiveShape::Pyramid,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id: window_main,
                geometry_id: self.ids.geometry_torus_id,
                label: Some("Demo 007 Torus".into()),
                shape: PrimitiveShape::Torus,
                options: None,
            }),
            EngineCmd::CmdMaterialUpsert(crate::core::cmd::CmdMaterialUpsertArgs::Create(
                CmdMaterialCreateArgs {
                    window_id: window_main,
                    material_id: self.ids.material_shape_id,
                    label: Some("Demo 007 Shape Material".into()),
                    kind: MaterialKind::Standard,
                    options: Some(MaterialOptions::Standard(StandardOptions {
                        base_color: Vec4::new(0.22, 0.82, 0.95, 1.0),
                        ..Default::default()
                    })),
                },
            )),
            create_ambient_light_cmd(window_main, 7110, Vec4::new(0.8, 0.85, 1.0, 1.0), 1.2),
            create_point_light_cmd(window_main, 7111, Vec4::new(4.0, 6.0, 3.0, 1.0)),
            EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
                CmdCameraCreateArgs {
                    camera_id: self.ids.camera_a_id,
                    label: Some("Demo 007 Camera A".into()),
                    transform: Mat4::look_at_rh(
                        Vec3::new(0.0, 3.0, 10.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    )
                        .inverse(),
                    kind: CameraKind::Perspective,
                    flags: 0,
                    near_far: Vec2::new(0.1, 120.0),
                    layer_mask: 0xFFFFFFFF,
                    order: 0,
                    view_position: None,
                    ortho_scale: 10.0,
                },
            )),
            EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
                CmdCameraCreateArgs {
                    camera_id: self.ids.camera_b_id,
                    label: Some("Demo 007 Camera B".into()),
                    transform: Mat4::look_at_rh(
                        Vec3::new(10.0, 3.0, 0.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    )
                    .inverse(),
                    kind: CameraKind::Perspective,
                    flags: 0,
                    near_far: Vec2::new(0.1, 120.0),
                    layer_mask: 0xFFFFFFFF,
                    order: 1,
                    view_position: None,
                    ortho_scale: 10.0,
                },
            )),
            EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
                CmdCameraCreateArgs {
                    camera_id: self.ids.camera_c_id,
                    label: Some("Demo 007 Camera C".into()),
                    transform: Mat4::look_at_rh(
                        Vec3::new(0.0, 3.0, -10.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    )
                    .inverse(),
                    kind: CameraKind::Perspective,
                    flags: 0,
                    near_far: Vec2::new(0.1, 120.0),
                    layer_mask: 0xFFFFFFFF,
                    order: 2,
                    view_position: None,
                    ortho_scale: 10.0,
                },
            )),
            EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
                CmdCameraCreateArgs {
                    camera_id: self.ids.camera_d_id,
                    label: Some("Demo 007 Camera D".into()),
                    transform: Mat4::look_at_rh(
                        Vec3::new(-10.0, 3.0, 0.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    )
                    .inverse(),
                    kind: CameraKind::Perspective,
                    flags: 0,
                    near_far: Vec2::new(0.1, 120.0),
                    layer_mask: 0xFFFFFFFF,
                    order: 3,
                    view_position: None,
                    ortho_scale: 10.0,
                },
            )),
        ];

        let model_geometries = [
            self.ids.geometry_cube_id,
            self.ids.geometry_cone_id,
            self.ids.geometry_torus_id,
        ];
        let model_positions = [
            Vec3::new(-4.0, 0.0, 0.0),
            Vec3::new(0.0, 0.2, -4.0),
            Vec3::new(4.0, 0.1, 2.5),
        ];
        let model_scales = [Vec3::splat(1.0), Vec3::new(1.1, 1.4, 1.1), Vec3::splat(1.2)];
        for (index, model_id) in self.ids.model_ids.iter().enumerate() {
            cmds.push(EngineCmd::CmdModelUpsert(
                crate::core::cmd::CmdModelUpsertArgs::Create(CmdModelCreateArgs {
                    window_id: window_main,
                    model_id: *model_id,
                    label: Some(format!("Demo 007 Shape {}", index + 1)),
                    geometry_id: model_geometries[index],
                    material_id: Some(self.ids.material_shape_id),
                    transform: Mat4::from_translation(model_positions[index])
                        * Mat4::from_scale(model_scales[index]),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: false,
                    outline_color: Vec4::ZERO,
                }),
            ));
        }

        assert_eq!(send_commands(cmds), VulframResult::Success);
        assert_command_batch_success(14, "setup");

        let ui_cmds = build_ui_cmds(self.ids, target_ids, realm_ui);
        assert_eq!(send_commands(ui_cmds), VulframResult::Success);
        assert_command_batch_success(3, "ui");

        Demo007RealmIds {
            _realm_ui: realm_ui,
            _realm_3d: realm_3d,
        }
    }
}

fn assert_command_batch_success(expected_responses: usize, tag: &str) {
    let mut received = 0usize;
    for _ in 0..120 {
        let responses = receive_responses();
        received += responses.len();
        for response in responses {
            match response.response {
                CommandResponse::EnvironmentUpsert(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::CameraUpsert(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::PrimitiveGeometryCreate(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::MaterialUpsert(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::LightUpsert(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::ModelUpsert(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::UiThemeDefine(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::UiDocumentCreate(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
                }
                CommandResponse::UiApplyOps(result) => {
                    assert!(result.success, "[demo007:{tag}] {}", result.message);
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
        "[demo007:{tag}] incomplete responses: got {received}, expected at least {expected_responses}"
    );
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
    for _ in 0..100 {
        let responses = receive_responses();
        for response in responses {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "Realm create failed: {}", result.message);
                return result.realm_id.expect("realm create missing id");
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(0, 0), VulframResult::Success);
    }
    panic!("Realm create did not complete");
}
