use crate::core::VulframResult;
use crate::core::audio::{
    CmdAudioListenerCreateArgs, CmdAudioResourceCreateArgs, CmdAudioResourcePushArgs,
    CmdAudioSourceCreateArgs,
};
use crate::core::cmd::EngineCmd;
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs,
    EnvironmentConfig, MsaaConfig, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode,
};
use crate::demo::DemoContext;
use crate::demo::demo_004::config::build_demo_004_post_config;
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{
    create_ambient_light_cmd, create_camera_cmd, create_floor_cmd, create_point_light_cmd,
    create_shadow_config_cmd, create_standard_material_cmd, load_texture_bytes,
    upload_binary_bytes,
};
use glam::{Mat4, Quat, Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Demo004Ids {
    pub geometry_id: u32,
    pub material_id: u32,
    pub floor_material_id: u32,
    pub emissive_material_id: u32,
    pub skybox_texture_id: u32,
    pub skybox_buffer_id: u64,
    pub audio_id: u32,
    pub audio_source_id: u32,
    pub audio_buffer_id: u64,
    pub listener_model_id: u32,
    pub emitter_geometry_id: u32,
    pub emitter_material_id: u32,
    pub emitter_model_id: u32,
    pub emitter_pos: Vec3,
    pub camera_id: u32,
}

pub struct Demo004Setup {
    pub ids: Demo004Ids,
    pub cube_models: Vec<(u32, Vec3, Vec4)>,
    pub post_config: PostProcessConfig,
    pub audio_chunk_ids: Vec<(u64, u64)>,
    pub audio_total_bytes: u64,
}

impl Demo004Setup {
    pub fn new() -> Self {
        let ids = Demo004Ids {
            geometry_id: 500,
            material_id: 502,
            floor_material_id: 503,
            emissive_material_id: 504,
            skybox_texture_id: 900,
            skybox_buffer_id: 9000,
            audio_id: 910,
            audio_source_id: 911,
            audio_buffer_id: 9100,
            listener_model_id: 920,
            emitter_geometry_id: 930,
            emitter_material_id: 931,
            emitter_model_id: 932,
            emitter_pos: Vec3::new(8.0, -5.2, 8.0),
            camera_id: 1,
        };

        let cube_models = vec![
            (
                501,
                Vec3::new(-2.5, 0.0, 0.0),
                Vec4::new(1.0, 0.1, 0.1, 1.0),
            ),
            (502, Vec3::new(0.0, 0.0, 0.0), Vec4::new(0.1, 1.0, 0.1, 1.0)),
            (503, Vec3::new(2.5, 0.0, 0.0), Vec4::new(0.1, 0.1, 1.0, 1.0)),
            (
                504,
                Vec3::new(-0.6, 0.2, -0.2),
                Vec4::new(1.0, 0.6, 0.1, 1.0),
            ),
            (505, Vec3::new(0.4, 0.1, 0.3), Vec4::new(0.6, 1.0, 0.9, 1.0)),
            (
                506,
                Vec3::new(0.0, -0.1, 0.8),
                Vec4::new(0.9, 0.4, 1.0, 1.0),
            ),
        ];

        let post_config = build_demo_004_post_config();

        let audio_bytes = load_texture_bytes("assets/audio.wav");
        let audio_chunk_size = 64 * 1024;
        let mut audio_chunk_ids = Vec::new();
        for (index, chunk) in audio_bytes.chunks(audio_chunk_size).enumerate() {
            let buffer_id = ids.audio_buffer_id + index as u64;
            upload_binary_bytes(chunk, buffer_id);
            audio_chunk_ids.push((buffer_id, index as u64 * audio_chunk_size as u64));
        }

        Self {
            ids,
            cube_models,
            post_config,
            audio_chunk_ids,
            audio_total_bytes: audio_bytes.len() as u64,
        }
    }

    pub fn apply(&self, ctx: DemoContext) {
        let window_id = ctx.window_id;
        let realm_id = ctx.realm_id;

        let setup_cmds = vec![
            EngineCmd::CmdEnvironmentUpsert(crate::core::cmd::CmdEnvironmentUpsertArgs::Update(
                CmdEnvironmentUpdateArgs {
                    environment_id: window_id,
                    config: EnvironmentConfig {
                        msaa: MsaaConfig {
                            enabled: true,
                            sample_count: 4,
                        },
                        skybox: SkyboxConfig {
                            mode: SkyboxMode::Cubemap,
                            intensity: 1.0,
                            rotation: 0.0,
                            ground_color: Vec3::new(1.0, 0.0, 0.0),
                            horizon_color: Vec3::new(1.00, 1.0, 1.0),
                            sky_color: Vec3::new(0.18, 0.32, 0.55),
                            cubemap_texture_id: Some(self.ids.skybox_texture_id),
                        },
                        clear_color: Vec3::new(0.0, 0.0, 0.0),
                        post: self.post_config.clone(),
                    },
                },
            )),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id: self.ids.geometry_id,
                label: Some("Graph Cube".into()),
                shape: PrimitiveShape::Cube,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id: self.ids.emitter_geometry_id,
                label: Some("Audio Emitter".into()),
                shape: PrimitiveShape::Sphere,
                options: None,
            }),
            create_camera_cmd(
                self.ids.camera_id,
                "Graph Camera",
                Mat4::look_at_rh(Vec3::new(0.0, 3.5, 8.0), Vec3::ZERO, Vec3::Y).inverse(),
            ),
            create_point_light_cmd(window_id, 2, Vec4::new(0.0, 5.0, 2.0, 1.0)),
            create_ambient_light_cmd(window_id, 3, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.6),
            create_standard_material_cmd(
                window_id,
                self.ids.material_id,
                "Graph Material",
                Vec4::ONE,
                None,
                None,
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.floor_material_id,
                "Graph Floor Material",
                Vec4::ONE,
                None,
                None,
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.emissive_material_id,
                "Graph Emissive Material",
                Vec4::ONE,
                None,
                Some(Vec4::new(5.0, 5.0, 5.0, 1.0)),
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.emitter_material_id,
                "Audio Emitter Material",
                Vec4::new(1.0, 0.8, 0.2, 1.0),
                None,
                Some(Vec4::new(2.5, 1.6, 0.4, 1.0)),
            ),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[0].0,
                    label: Some("Graph Cube R".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(self.cube_models[0].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[0].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[1].0,
                    label: Some("Graph Cube G".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(self.cube_models[1].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[1].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[2].0,
                    label: Some("Graph Cube B".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.emissive_material_id),
                    transform: Mat4::from_translation(self.cube_models[2].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[2].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[3].0,
                    label: Some("Graph Cube D".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(self.cube_models[3].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[3].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[4].0,
                    label: Some("Graph Cube E".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(self.cube_models[4].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[4].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.cube_models[5].0,
                    label: Some("Graph Cube F".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(self.cube_models[5].1),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: true,
                    receive_shadow: true,
                    cast_outline: true,
                    outline_color: self.cube_models[5].2,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.ids.listener_model_id,
                    label: Some("Audio Listener".into()),
                    geometry_id: self.ids.geometry_id,
                    material_id: Some(self.ids.material_id),
                    transform: Mat4::from_translation(Vec3::new(0.0, 3.5, 8.0)),
                    layer_mask: 0,
                    cast_shadow: false,
                    receive_shadow: false,
                    cast_outline: false,
                    outline_color: Vec4::ZERO,
                },
            )),
            EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
                CmdModelCreateArgs {
                    window_id,
                    model_id: self.ids.emitter_model_id,
                    label: Some("Audio Emitter Sphere".into()),
                    geometry_id: self.ids.emitter_geometry_id,
                    material_id: Some(self.ids.emitter_material_id),
                    transform: Mat4::from_translation(self.ids.emitter_pos)
                        * Mat4::from_scale(Vec3::splat(0.6)),
                    layer_mask: 0xFFFFFFFF,
                    cast_shadow: false,
                    receive_shadow: true,
                    cast_outline: false,
                    outline_color: Vec4::ZERO,
                },
            )),
            create_floor_cmd(window_id, self.ids.geometry_id, self.ids.floor_material_id),
            create_shadow_config_cmd(window_id),
            EngineCmd::CmdAudioListenerUpsert(
                crate::core::cmd::CmdAudioListenerUpsertArgs::Create(CmdAudioListenerCreateArgs {
                    realm_id,
                    model_id: self.ids.listener_model_id,
                }),
            ),
            EngineCmd::CmdAudioResourceCreate(CmdAudioResourceCreateArgs {
                resource_id: self.ids.audio_id,
                buffer_id: self
                    .audio_chunk_ids
                    .first()
                    .map(|(buffer_id, _)| *buffer_id)
                    .unwrap_or(self.ids.audio_buffer_id),
                total_bytes: Some(self.audio_total_bytes),
                offset_bytes: Some(0),
            }),
            EngineCmd::CmdAudioSourceUpsert(crate::core::cmd::CmdAudioSourceUpsertArgs::Create(
                CmdAudioSourceCreateArgs {
                    realm_id,
                    source_id: self.ids.audio_source_id,
                    model_id: self.ids.emitter_model_id,
                    position: self.ids.emitter_pos,
                    velocity: Vec3::ZERO,
                    orientation: Quat::IDENTITY,
                    gain: 1.0,
                    pitch: 1.0,
                    spatial: crate::core::audio::AudioSpatialParamsDto::default(),
                },
            )),
        ];

        assert_eq!(send_commands(setup_cmds), VulframResult::Success);
        if self.audio_chunk_ids.len() > 1 {
            let mut chunk_cmds = Vec::new();
            for (buffer_id, offset_bytes) in self.audio_chunk_ids.iter().skip(1) {
                chunk_cmds.push(EngineCmd::CmdAudioResourcePush(CmdAudioResourcePushArgs {
                    resource_id: self.ids.audio_id,
                    buffer_id: *buffer_id,
                    offset_bytes: *offset_bytes,
                }));
            }
            assert_eq!(send_commands(chunk_cmds), VulframResult::Success);
        }
        let _ = receive_responses();
    }
}
