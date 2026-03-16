use crate::core::cmd::EngineCmd;
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs, MaterialKind,
    MaterialOptions, MaterialSampler, PbrOptions, StandardOptions,
};
use glam::{Mat4, Vec2, Vec4};

pub fn create_camera_cmd(realm_id: u32, camera_id: u32, label: &str, transform: Mat4) -> EngineCmd {
    EngineCmd::CmdCameraUpsert(crate::core::cmd::CmdCameraUpsertArgs::Create(
        CmdCameraCreateArgs {
            realm_id,
            camera_id,
            label: Some(label.to_string()),
            transform,
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        },
    ))
}

pub fn create_point_light_cmd(realm_id: u32, light_id: u32, position: Vec4) -> EngineCmd {
    EngineCmd::CmdLightUpsert(crate::core::cmd::CmdLightUpsertArgs::Create(
        CmdLightCreateArgs {
            realm_id,
            light_id,
            label: Some("Point Light".to_string()),
            kind: Some(crate::core::resources::LightKind::Point),
            position: Some(position),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(20.0),
            range: Some(30.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        },
    ))
}

pub fn create_ambient_light_cmd(
    realm_id: u32,
    light_id: u32,
    color: Vec4,
    intensity: f32,
) -> EngineCmd {
    EngineCmd::CmdLightUpsert(crate::core::cmd::CmdLightUpsertArgs::Create(
        CmdLightCreateArgs {
            realm_id,
            light_id,
            label: Some("Ambient Light".to_string()),
            kind: Some(crate::core::resources::LightKind::Ambient),
            position: None,
            direction: None,
            color: Some(color),
            ground_color: None,
            intensity: Some(intensity),
            range: Some(1.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
        },
    ))
}

pub fn create_standard_material_cmd(
    material_id: u32,
    label: &str,
    base_color: Vec4,
    base_tex_id: Option<u32>,
    emissive_color: Option<Vec4>,
) -> EngineCmd {
    EngineCmd::CmdMaterialUpsert(crate::core::cmd::CmdMaterialUpsertArgs::Create(
        CmdMaterialCreateArgs {
            material_id,
            label: Some(label.to_string()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color,
                base_tex_id,
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: emissive_color.unwrap_or(Vec4::ZERO),
                ..Default::default()
            })),
        },
    ))
}

pub fn create_pbr_material_cmd(
    material_id: u32,
    label: &str,
    base_color: Vec4,
    metallic: f32,
    roughness: f32,
) -> EngineCmd {
    EngineCmd::CmdMaterialUpsert(crate::core::cmd::CmdMaterialUpsertArgs::Create(
        CmdMaterialCreateArgs {
            material_id,
            label: Some(label.to_string()),
            kind: MaterialKind::Pbr,
            options: Some(MaterialOptions::Pbr(PbrOptions {
                base_color,
                metallic,
                roughness,
                ..Default::default()
            })),
        },
    ))
}

pub fn create_shadow_config_cmd(window_id: u32) -> EngineCmd {
    EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
        window_id,
        config: ShadowConfig::default(),
    })
}
