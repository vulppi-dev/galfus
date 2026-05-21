use std::time::{Duration, Instant};

use galfus_core::core;
use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::EngineEvent;
use galfus_core::core::cmd::{
    CmdCameraUpsertArgs, CmdEnvironmentUpsertArgs, CmdLightUpsertArgs,
    CmdMaterialDefinitionUpsertArgs, CmdMaterialUpsertArgs, CmdModelUpsertArgs, EngineCmd,
};
use galfus_core::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdLightCreateArgs,
    CmdMaterialCreateArgs, CmdMaterialDefinitionCreateArgs, CmdModelCreateArgs, CmdModelUpdateArgs,
    CmdPrimitiveGeometryCreateArgs, EnvironmentConfig, LightKind, MaterialKind, MaterialOptions,
    MaterialShaderCapabilities, MaterialShaderType, PostProcessConfig, PrimitiveShape, RenderSide,
    ShaderMaterialPreset, StandardOptions,
};
use galfus_core::core::target::{
    CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs, DimensionValue, TargetKind, TargetLayerLayout,
};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::demo::DemoContext;
use crate::demo::io::{receive_events, receive_responses, send_commands};

const FRAME_MS: u32 = 16;
const RUN_DURATION: Duration = Duration::from_secs(8);

const WINDOW_TARGET_ID: u64 = 1;
const GEOMETRY_CUBE_ID: u32 = 201;
const GEOMETRY_FLOOR_ID: u32 = 202;
const MATERIAL_DEF_PERSISTENCE_ID: u32 = 210;
const MATERIAL_PERSISTENCE_ID: u32 = 211;
const MATERIAL_FLOOR_ID: u32 = 212;
const CAMERA_ID: u32 = 220;
const LIGHT_ID: u32 = 221;
const ENVIRONMENT_ID: u32 = 222;
const MODEL_CUBE_ID: u32 = 230;
const MODEL_FLOOR_ID: u32 = 231;

pub fn run(ctx: DemoContext) -> bool {
    let realm_3d = ctx.realm_id;
    let mut setup = vec![EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
        target_id: WINDOW_TARGET_ID,
        kind: TargetKind::Window,
        window_id: Some(ctx.window_id),
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    })];
    setup.extend(build_scene(realm_3d));
    setup.push(bind_layer(
        realm_3d,
        WINDOW_TARGET_ID,
        full_layout(0),
        vec![CAMERA_ID],
        Some(ENVIRONMENT_ID),
    ));
    let _ = send_commands(setup);

    let mut total_ms: u64 = 0;
    let start = Instant::now();
    while start.elapsed() < RUN_DURATION {
        let t = total_ms as f32 / 1000.0;
        let updates = vec![EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Update(
            CmdModelUpdateArgs {
                realm_id: realm_3d,
                model_id: MODEL_CUBE_ID,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_translation(Vec3::new(t.sin() * 2.5, 0.8 + (t * 1.7).sin() * 0.2, 0.0))
                        * Mat4::from_rotation_y(t * 2.1)
                        * Mat4::from_rotation_x(t * 1.1),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            },
        ))];
        let _ = send_commands(updates);
        assert_eq!(core::galfus_tick(total_ms, FRAME_MS), GalfusResult::Success);
        total_ms = total_ms.saturating_add(FRAME_MS as u64);
        let _ = receive_responses();
        print_runtime_logs();
        std::thread::sleep(Duration::from_millis(FRAME_MS as u64));
    }

    false
}

fn build_scene(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_CUBE_ID,
            label: Some("demo2-cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_FLOOR_ID,
            label: Some("demo2-floor".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs::Create(
            CmdMaterialDefinitionCreateArgs {
                definition_id: MATERIAL_DEF_PERSISTENCE_ID,
                slug: "demo2-persistence".into(),
                label: Some("demo2-persistence-definition".into()),
                preset: ShaderMaterialPreset::Standard,
                shader_type: Some(MaterialShaderType::Model),
                shader_source: r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let base = vec3<f32>(0.12, 0.72, 1.0);
  let uv = input.uv;
  let history = sample_history0(uv).rgb;
  let fresnel = pow(1.0 - max(dot(normalize(input.world_normal), normalize(camera.position.xyz - input.world_position)), 0.0), 2.0);
  let live = mix(base, vec3<f32>(1.0, 1.0, 1.0), fresnel * 0.5);
  let persisted = history * 0.90;
  out.color = vec4<f32>(max(live, persisted), 1.0);
  out.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  return out;
}
"#
                .to_string(),
                shader_params_schema: None,
                capabilities: Some(MaterialShaderCapabilities {
                    semantics: vec!["history0".to_string()],
                }),
            },
        )),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_PERSISTENCE_ID,
            label: Some("demo2-mat-persistence".into()),
            slug: "demo2-persistence".into(),
            kind: MaterialKind::Shader,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Some(Vec4::new(0.2, 0.7, 1.0, 1.0)),
                render_side: Some(RenderSide::Back),
                ..Default::default()
            })),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_FLOOR_ID,
            label: Some("demo2-mat-floor".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Some(Vec4::new(0.08, 0.08, 0.10, 1.0)),
                spec_color: Some(Vec4::new(0.02, 0.02, 0.02, 1.0)),
                spec_power: Some(6.0),
                render_side: Some(RenderSide::DoubleSide),
                ..Default::default()
            })),
        })),
        EngineCmd::CmdCameraUpsert(CmdCameraUpsertArgs::Create(CmdCameraCreateArgs {
            realm_id,
            camera_id: CAMERA_ID,
            label: Some("demo2-camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 2.3, 7.5), Vec3::new(0.0, 0.7, 0.0), Vec3::Y)
                .inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 120.0),
            layer_mask: 1,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        })),
        EngineCmd::CmdLightUpsert(CmdLightUpsertArgs::Create(CmdLightCreateArgs {
            realm_id,
            light_id: LIGHT_ID,
            label: Some("demo2-light".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(3.0, 4.0, 4.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(3.5),
            range: Some(24.0),
            spot_inner_outer: None,
            layer_mask: 1,
            cast_shadow: true,
        })),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Create(
            CmdEnvironmentCreateArgs {
                environment_id: ENVIRONMENT_ID,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    post: PostProcessConfig {
                        outline_enabled: false,
                        bloom_enabled: false,
                        ssao_enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_ID,
            label: Some("demo2-cube-model".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_PERSISTENCE_ID),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.8, 0.0)),
            layer_mask: 1,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_FLOOR_ID,
            label: Some("demo2-floor-model".into()),
            geometry_id: GEOMETRY_FLOOR_ID,
            material_id: Some(MATERIAL_FLOOR_ID),
            transform: Mat4::from_scale(Vec3::new(12.0, 1.0, 12.0))
                * Mat4::from_translation(Vec3::new(0.0, -0.01, 0.0)),
            layer_mask: 1,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
    ]
}

fn bind_layer(
    realm_id: u32,
    target_id: u64,
    layout: TargetLayerLayout,
    enabled_camera_ids: Vec<u32>,
    environment_id: Option<u32>,
) -> EngineCmd {
    EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id,
        target_id,
        layout,
        enabled_camera_ids,
        environment_id,
    })
}

fn full_layout(z_index: i32) -> TargetLayerLayout {
    TargetLayerLayout {
        left: DimensionValue::Percent(0.0),
        top: DimensionValue::Percent(0.0),
        width: DimensionValue::Percent(100.0),
        height: DimensionValue::Percent(100.0),
        enabled: true,
        opacity: 1.0,
        z_index,
        blend_mode: 0,
        clip: None,
    }
}

fn print_runtime_logs() {
    for event in receive_events() {
        if let EngineEvent::Log(log) = event {
            println!("[runtime/{:?}][{}] {}", log.level, log.tag, log.message);
        }
    }
}
