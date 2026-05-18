use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};
use vulfram_core::core;
use vulfram_core::core::VulframResult;
use vulfram_core::core::cmd::EngineEvent;
use vulfram_core::core::cmd::{
    CmdCameraUpsertArgs, CmdEnvironmentUpsertArgs, CmdLightUpsertArgs, CmdMaterialUpsertArgs,
    CmdModelUpsertArgs, EngineCmd,
};
use vulfram_core::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdLightCreateArgs,
    CmdMaterialCreateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs, EnvironmentConfig,
    LightKind, MaterialKind, MaterialOptions, PrimitiveShape, ShaderMaterialPreset,
    StandardOptions,
};
use vulfram_core::core::target::{
    CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs, DimensionValue, TargetKind, TargetLayerLayout,
};

use crate::demo::DemoContext;
use crate::demo::io::{receive_events, receive_responses, send_commands};

const FRAME_MS: u32 = 16;
const RUN_DURATION: Duration = Duration::from_secs(6);

const WINDOW_TARGET_ID: u64 = 1;

const GEOMETRY_CUBE_ID: u32 = 1001;
const MATERIAL_STANDARD_ID: u32 = 2001;
const MATERIAL_PBR_ID: u32 = 2002;
const MATERIAL_CUSTOM_SIMPLE_ID: u32 = 2003;
const CAMERA_ID: u32 = 3001;
const LIGHT_ID: u32 = 4001;
const ENVIRONMENT_ID: u32 = 4501;
const MODEL_CUBE_A_ID: u32 = 5001;
const MODEL_CUBE_B_ID: u32 = 5002;
const MODEL_CUBE_C_ID: u32 = 5003;

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

    setup.extend(build_realm3d_scene(realm_3d));
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
        assert_eq!(
            core::vulfram_tick(total_ms, FRAME_MS),
            VulframResult::Success
        );
        total_ms = total_ms.saturating_add(FRAME_MS as u64);
        let _ = receive_responses();
        print_runtime_logs();
        std::thread::sleep(Duration::from_millis(FRAME_MS as u64));
    }

    false
}

fn print_runtime_logs() {
    for event in receive_events() {
        if let EngineEvent::Log(log) = event {
            println!("[runtime/{:?}][{}] {}", log.level, log.tag, log.message);
        }
    }
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

fn build_realm3d_scene(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_CUBE_ID,
            label: Some("demo-cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_STANDARD_ID,
            label: Some("demo-mat-standard".into()),
            kind: MaterialKind::Shader,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_source: None,
            shader_params_schema: None,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Some(Vec4::new(0.92, 0.35, 0.32, 1.0)),
                ..Default::default()
            })),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_PBR_ID,
            label: Some("demo-mat-pbr".into()),
            kind: MaterialKind::Shader,
            preset: Some(ShaderMaterialPreset::Pbr),
            shader_source: None,
            shader_params_schema: None,
            options: Some(MaterialOptions::Pbr(
                vulfram_core::core::resources::PbrOptions {
                    base_color: Some(Vec4::new(0.25, 0.86, 0.62, 1.0)),
                    metallic: Some(0.55),
                    roughness: Some(0.35),
                    ..Default::default()
                },
            )),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_CUSTOM_SIMPLE_ID,
            label: Some("demo-mat-custom-simple".into()),
            kind: MaterialKind::Shader,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_source: None,
            shader_params_schema: None,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Some(Vec4::new(0.32, 0.46, 0.98, 1.0)),
                emissive_color: Some(Vec4::new(0.04, 0.08, 0.2, 0.0)),
                spec_color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
                spec_power: Some(48.0),
                ..Default::default()
            })),
        })),
        EngineCmd::CmdCameraUpsert(CmdCameraUpsertArgs::Create(CmdCameraCreateArgs {
            realm_id,
            camera_id: CAMERA_ID,
            label: Some("demo-camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 2.0, 7.0), Vec3::ZERO, Vec3::Y).inverse(),
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
            label: Some("demo-light".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(3.0, 5.0, 5.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(10.0),
            range: Some(30.0),
            spot_inner_outer: None,
            layer_mask: 1,
            cast_shadow: false,
        })),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Create(
            CmdEnvironmentCreateArgs {
                environment_id: ENVIRONMENT_ID,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.03, 0.03, 0.04, 1.0),
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_A_ID,
            label: Some("demo-cube-standard".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_STANDARD_ID),
            transform: Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
            layer_mask: 1,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_B_ID,
            label: Some("demo-cube-pbr".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_PBR_ID),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            layer_mask: 1,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_C_ID,
            label: Some("demo-cube-custom-simple".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_CUSTOM_SIMPLE_ID),
            transform: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            layer_mask: 1,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
    ]
}
