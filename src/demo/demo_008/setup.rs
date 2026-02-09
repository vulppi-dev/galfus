use std::time::Duration;

use glam::{Mat4, Quat, Vec3, Vec4};

use crate::core::VulframResult;
use crate::core::audio::{
    CmdAudioListenerCreateArgs, CmdAudioResourceCreateArgs, CmdAudioResourcePushArgs,
    CmdAudioSourceCreateArgs,
};
use crate::core::cmd::{CommandResponse, EngineCmd};
use crate::core::realm::cmd::{
    CmdConnectorCreateArgs, CmdRealmCreateArgs, CmdSurfaceCreateArgs, RealmKindDto,
    SurfaceKindDto,
};
use crate::core::render::cmd::{CmdRenderGraph2DSetArgs, CmdRenderGraph3DSetArgs};
use crate::core::render::graph::RenderGraphDesc;
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs, EnvironmentConfig,
    MsaaConfig, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode,
};
use crate::demo::demo_008::graph::build_demo_008_graph;
use crate::demo::{
    create_ambient_light_cmd, create_camera_cmd, create_floor_cmd, create_point_light_cmd,
    create_shadow_config_cmd, create_standard_material_cmd, load_texture_bytes,
    upload_binary_bytes, DemoContext,
};
use crate::demo::io::{receive_responses, send_commands};

#[derive(Debug, Clone, Copy)]
pub struct Demo008Ids {
    pub cube_geometry_id: u32,
    pub floor_geometry_id: u32,
    pub emitter_geometry_id: u32,
    pub material_primary_id: u32,
    pub material_accent_id: u32,
    pub material_floor_id: u32,
    pub material_emitter_id: u32,
    pub listener_model_id: u32,
    pub emitter_model_id: u32,
    pub camera_id: u32,
    pub audio_id: u32,
    pub audio_source_id: u32,
    pub audio_buffer_id: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Demo008RealmIds {
    pub realm_a: u32,
    pub realm_b: u32,
    pub realm_ui: u32,
    pub surface_a: u32,
    pub surface_b: u32,
    pub surface_ui: u32,
    pub connector_a: u32,
    pub connector_b: u32,
    pub connector_cycle: u32,
    pub connector_self: u32,
}

pub struct Demo008Setup {
    pub ids: Demo008Ids,
    pub graph: RenderGraphDesc,
    pub post_config: PostProcessConfig,
    pub audio_chunk_ids: Vec<(u64, u64)>,
    pub audio_total_bytes: u64,
}

impl Demo008Setup {
    pub fn new() -> Self {
        let ids = Demo008Ids {
            cube_geometry_id: 800,
            floor_geometry_id: 801,
            emitter_geometry_id: 802,
            material_primary_id: 810,
            material_accent_id: 811,
            material_floor_id: 812,
            material_emitter_id: 813,
            listener_model_id: 820,
            emitter_model_id: 821,
            camera_id: 1,
            audio_id: 830,
            audio_source_id: 831,
            audio_buffer_id: 8300,
        };

        let graph = build_demo_008_graph();
        let post_config = build_demo_008_post_config();

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
            graph,
            post_config,
            audio_chunk_ids,
            audio_total_bytes: audio_bytes.len() as u64,
        }
    }

    pub fn apply(&self, ctx: DemoContext) -> Demo008RealmIds {
        let window_id = ctx.window_id;
        let realm_id = ctx.realm_id;

        let surface_a = create_surface(
            SurfaceKindDto::Offscreen,
            glam::UVec2::new(640, 360),
        );
        let surface_b = create_surface(
            SurfaceKindDto::Offscreen,
            glam::UVec2::new(320, 240),
        );
        let surface_ui = create_surface(
            SurfaceKindDto::Offscreen,
            glam::UVec2::new(512, 512),
        );

        let realm_a = create_realm(
            RealmKindDto::ThreeD,
            surface_a,
            Some(window_id),
        );
        let realm_b = create_realm(
            RealmKindDto::ThreeD,
            surface_b,
            Some(window_id),
        );
        let realm_ui = create_realm(
            RealmKindDto::TwoD,
            surface_ui,
            Some(window_id),
        );

        let connector_a = create_connector(
            realm_id,
            surface_a,
            Vec4::new(40.0, 40.0, 280.0, 220.0),
            1,
            0,
            None,
        );
        let connector_b = create_connector(
            realm_id,
            surface_b,
            Vec4::new(720.0, 120.0, 200.0, 180.0),
            2,
            1,
            Some(Vec4::new(720.0, 120.0, 160.0, 140.0)),
        );
        let connector_cycle = create_connector(
            realm_a,
            ctx.surface_id,
            Vec4::new(60.0, 360.0, 220.0, 160.0),
            0,
            0,
            None,
        );
        let connector_self = create_connector(
            realm_id,
            ctx.surface_id,
            Vec4::new(1020.0, 40.0, 140.0, 100.0),
            -1,
            0,
            None,
        );

        let setup_cmds = vec![
            EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
                window_id,
                config: EnvironmentConfig {
                    msaa: MsaaConfig {
                        enabled: true,
                        sample_count: 4,
                    },
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::Procedural,
                        intensity: 1.0,
                        rotation: 0.0,
                        ground_color: Vec3::new(0.02, 0.02, 0.03),
                        horizon_color: Vec3::new(0.2, 0.2, 0.35),
                        sky_color: Vec3::new(0.18, 0.32, 0.55),
                        cubemap_texture_id: None,
                    },
                    post: self.post_config.clone(),
                },
            }),
            EngineCmd::CmdRenderGraph3DSet(CmdRenderGraph3DSetArgs {
                realm_id,
                graph: self.graph.clone(),
            }),
            EngineCmd::CmdRenderGraph3DSet(CmdRenderGraph3DSetArgs {
                realm_id: realm_a,
                graph: self.graph.clone(),
            }),
            EngineCmd::CmdRenderGraph3DSet(CmdRenderGraph3DSetArgs {
                realm_id: realm_b,
                graph: self.graph.clone(),
            }),
            EngineCmd::CmdRenderGraph2DSet(CmdRenderGraph2DSetArgs {
                realm_id: realm_ui,
                graph: self.graph.clone(),
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id: self.ids.cube_geometry_id,
                label: Some("Demo 008 Cube".into()),
                shape: PrimitiveShape::Cube,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id: self.ids.floor_geometry_id,
                label: Some("Demo 008 Floor".into()),
                shape: PrimitiveShape::Plane,
                options: None,
            }),
            EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id: self.ids.emitter_geometry_id,
                label: Some("Demo 008 Emitter".into()),
                shape: PrimitiveShape::Sphere,
                options: None,
            }),
            create_camera_cmd(
                self.ids.camera_id,
                "Demo 008 Camera",
                Mat4::look_at_rh(Vec3::new(0.0, 3.0, 9.0), Vec3::ZERO, Vec3::Y).inverse(),
            ),
            create_point_light_cmd(window_id, 2, Vec4::new(3.0, 6.0, 2.0, 1.0)),
            create_ambient_light_cmd(window_id, 3, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.7),
            create_standard_material_cmd(
                window_id,
                self.ids.material_primary_id,
                "Demo 008 Primary",
                Vec4::new(0.2, 0.6, 0.9, 1.0),
                None,
                None,
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.material_accent_id,
                "Demo 008 Accent",
                Vec4::new(0.9, 0.4, 0.2, 1.0),
                None,
                None,
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.material_floor_id,
                "Demo 008 Floor",
                Vec4::new(0.4, 0.4, 0.45, 1.0),
                None,
                None,
            ),
            create_standard_material_cmd(
                window_id,
                self.ids.material_emitter_id,
                "Demo 008 Emitter",
                Vec4::new(1.0, 0.85, 0.2, 1.0),
                None,
                Some(Vec4::new(2.5, 1.8, 0.6, 1.0)),
            ),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id,
                model_id: 840,
                label: Some("Demo 008 Cube A".into()),
                geometry_id: self.ids.cube_geometry_id,
                material_id: Some(self.ids.material_primary_id),
                transform: Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.8, 0.2, 0.2, 1.0),
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id,
                model_id: 841,
                label: Some("Demo 008 Cube B".into()),
                geometry_id: self.ids.cube_geometry_id,
                material_id: Some(self.ids.material_accent_id),
                transform: Mat4::from_translation(Vec3::new(2.2, 0.2, -1.0))
                    * Mat4::from_scale(Vec3::splat(1.2)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.2, 0.8, 0.4, 1.0),
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id,
                model_id: self.ids.listener_model_id,
                label: Some("Demo 008 Listener".into()),
                geometry_id: self.ids.cube_geometry_id,
                material_id: Some(self.ids.material_primary_id),
                transform: Mat4::from_translation(Vec3::new(0.0, 3.0, 9.0))
                    * Mat4::from_scale(Vec3::splat(0.4)),
                layer_mask: 0,
                cast_shadow: false,
                receive_shadow: false,
                cast_outline: false,
                outline_color: Vec4::ZERO,
            }),
            EngineCmd::CmdModelCreate(CmdModelCreateArgs {
                window_id,
                model_id: self.ids.emitter_model_id,
                label: Some("Demo 008 Emitter".into()),
                geometry_id: self.ids.emitter_geometry_id,
                material_id: Some(self.ids.material_emitter_id),
                transform: Mat4::from_translation(Vec3::new(4.5, 0.5, 3.5))
                    * Mat4::from_scale(Vec3::splat(0.5)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: false,
                receive_shadow: true,
                cast_outline: false,
                outline_color: Vec4::ZERO,
            }),
            create_floor_cmd(window_id, self.ids.floor_geometry_id, self.ids.material_floor_id),
            create_shadow_config_cmd(window_id),
            EngineCmd::CmdAudioListenerCreate(CmdAudioListenerCreateArgs {
                realm_id,
                model_id: self.ids.listener_model_id,
            }),
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
            EngineCmd::CmdAudioSourceCreate(CmdAudioSourceCreateArgs {
                realm_id,
                source_id: self.ids.audio_source_id,
                model_id: self.ids.emitter_model_id,
                position: Vec3::new(4.5, 0.5, 3.5),
                velocity: Vec3::ZERO,
                orientation: Quat::IDENTITY,
                gain: 1.0,
                pitch: 1.0,
                spatial: crate::core::audio::AudioSpatialParamsDto::default(),
            }),
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

        let responses = receive_responses();
        for response in responses {
            match response.response {
                CommandResponse::RenderGraph3DSet(result) => {
                    println!(
                        "RenderGraph3DSet: success={} fallback={} message={}",
                        result.success, result.fallback_used, result.message
                    );
                }
                CommandResponse::RenderGraph2DSet(result) => {
                    println!(
                        "RenderGraph2DSet: success={} fallback={} message={}",
                        result.success, result.fallback_used, result.message
                    );
                }
                _ => {}
            }
        }

        Demo008RealmIds {
            realm_a,
            realm_b,
            realm_ui,
            surface_a,
            surface_b,
            surface_ui,
            connector_a,
            connector_b,
            connector_cycle,
            connector_self,
        }
    }
}

fn build_demo_008_post_config() -> PostProcessConfig {
    PostProcessConfig {
        filter_enabled: true,
        filter_exposure: 1.0,
        filter_gamma: 2.2,
        filter_saturation: 1.05,
        filter_contrast: 1.05,
        filter_vignette: 0.1,
        filter_grain: 0.05,
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

fn create_surface(kind: SurfaceKindDto, size: glam::UVec2) -> u32 {
    assert_eq!(
        send_commands(vec![EngineCmd::CmdSurfaceCreate(CmdSurfaceCreateArgs {
            kind,
            size,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        })]),
        VulframResult::Success
    );
    wait_for_response(|response| match response {
        CommandResponse::SurfaceCreate(result) if result.success => result.surface_id,
        _ => None,
    })
    .expect("surface creation failed")
}

fn create_realm(kind: RealmKindDto, surface_id: u32, host_window_id: Option<u32>) -> u32 {
    assert_eq!(
        send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind,
            output_surface_id: Some(surface_id),
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

fn create_connector(
    target_realm_id: u32,
    source_surface_id: u32,
    rect: Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<Vec4>,
) -> u32 {
    assert_eq!(
        send_commands(vec![EngineCmd::CmdConnectorCreate(CmdConnectorCreateArgs {
            target_realm_id,
            source_surface_id,
            rect,
            z_index,
            blend_mode,
            clip,
            input_flags: 0,
        })]),
        VulframResult::Success
    );
    wait_for_response(|response| match response {
        CommandResponse::ConnectorCreate(result) if result.success => result.connector_id,
        _ => None,
    })
    .expect("connector creation failed")
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
