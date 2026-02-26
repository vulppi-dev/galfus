use std::time::Duration;

use glam::{Mat4, UVec2, Vec2, Vec3, Vec4};

use crate::core;
use crate::core::audio::cmd::{
    AudioPlayModeDto, AudioSourceTransportActionDto, AudioSpatialParamsDto,
    CmdAudioResourceUpsertArgs, CmdAudioSourceCreateArgs, CmdAudioSourceTransportArgs,
};
use crate::core::buffers::cmd::CmdUploadBufferDiscardAllArgs;
use crate::core::cmd::{
    CmdCameraUpsertArgs, CmdEnvironmentUpsertArgs, CmdGeometryUpsertArgs, CmdLightUpsertArgs,
    CmdMaterialUpsertArgs, CmdModelUpsertArgs, CommandResponse, EngineCmd, EngineEvent,
};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::profiling::state::ProfilingDetailLevel;
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::render::gizmos::{CmdGizmoDrawAabbArgs, CmdGizmoDrawLineArgs};
use crate::core::resources::geometry::{CmdGeometryCreateArgs, GeometryPrimitiveEntry};
use crate::core::resources::shadow::CmdShadowConfigureArgs;
use crate::core::resources::shadow::ShadowConfig;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdCameraDisposeArgs, CmdCameraListArgs,
    CmdEnvironmentDisposeArgs, CmdEnvironmentUpdateArgs, CmdGeometryDisposeArgs,
    CmdGeometryListArgs, CmdLightCreateArgs, CmdLightDisposeArgs, CmdLightListArgs,
    CmdMaterialCreateArgs, CmdMaterialDisposeArgs, CmdMaterialListArgs, CmdModelCreateArgs,
    CmdModelListArgs, CmdModelUpdateArgs, CmdPoseUpdateArgs, CmdPrimitiveGeometryCreateArgs,
    CmdTextureBindTargetArgs, CmdTextureCreateFromBufferArgs, CmdTextureCreateSolidColorArgs,
    CmdTextureDisposeArgs, CmdTextureListArgs, EnvironmentConfig, LightKind, MaterialKind,
    MaterialOptions, MsaaConfig, PbrOptions, PostProcessConfig, PrimitiveShape, SkyboxConfig,
    SkyboxMode, TextureCreateMode,
};
use crate::core::system::{
    diagnostics::CmdSystemDiagnosticsSetArgs,
    notification::{CmdNotificationSendArgs, NotificationLevel},
};
use crate::core::target::{
    DimensionValue, TargetKind, TargetLayerLayout,
    cmd::{
        CmdTargetDisposeArgs, CmdTargetLayerDisposeArgs, CmdTargetLayerUpsertArgs,
        CmdTargetUpsertArgs,
    },
};
use crate::core::ui::cmd::{
    CmdUiAccessKitActionRequestArgs, CmdUiClipboardPasteArgs, CmdUiDebugSetArgs,
    CmdUiDocumentCreateArgs, CmdUiDocumentGetLayoutRectsArgs, CmdUiDocumentGetTreeArgs,
    CmdUiEventTraceSetArgs, CmdUiFocusGetArgs, CmdUiFocusSetArgs, CmdUiImageCreateFromBufferArgs,
    CmdUiImageDisposeArgs, CmdUiScreenshotReplyArgs,
};
use crate::core::ui::types::{UiNode, UiNodeKind, UiNodeProps, UiOp};
use crate::core::window::{
    CmdWindowCursorArgs, CmdWindowMeasurementArgs, CmdWindowStateArgs, CursorIcon,
    EngineWindowState,
};
use crate::demo::assets::{
    load_texture_bytes, upload_binary_bytes, upload_buffer, upload_texture_bytes,
};
use crate::demo::commands::{
    create_ambient_light_cmd, create_camera_cmd, create_point_light_cmd, create_shadow_config_cmd,
    create_standard_material_cmd,
};
use crate::demo::geometry::build_skinned_plane;
use crate::demo::hud::FpsHud;
use crate::demo::io::{receive_events, receive_responses, send_commands};
use crate::demo::loop_utils::run_loop_with_events;
use crate::demo::session::create_window;
use crate::demo::{DemoContext, DemoKind};

#[derive(Clone, Copy)]
struct DemoIds {
    camera_id: u32,
    geometry_id: u32,
    material_id: u32,
    model_id: u32,
    light_id: u32,
    target_id: u64,
    texture_id: u32,
    env_id: u32,
    aux_id: u32,
    ui_doc_extra: u32,
    ui_node_extra: u32,
}

impl DemoIds {
    fn from_number(number: u32) -> Self {
        let base = number * 100;
        Self {
            camera_id: base + 1,
            geometry_id: base + 2,
            material_id: base + 3,
            model_id: base + 4,
            light_id: base + 5,
            target_id: 50_000 + number as u64,
            texture_id: base + 6,
            env_id: base + 7,
            aux_id: base + 8,
            ui_doc_extra: 95_000 + number,
            ui_node_extra: 96_000 + number,
        }
    }
}

pub fn run(demo: DemoKind, ctx: DemoContext) -> bool {
    let demo_number = demo.number();
    let ids = DemoIds::from_number(demo_number);
    let ui_realm_id = create_ui_realm(ctx.window_id).unwrap_or(ctx.realm_id);
    if demo == DemoKind::Demo003 {
        _ = create_and_dispose_temp_realm(ctx.window_id);
    }

    let mut hud = FpsHud::new(demo_number);
    let mut setup_cmds = base_scene_commands(ctx, ids);
    setup_cmds.extend(hud.setup_commands(ui_realm_id));

    let mut aux_windows: Vec<u32> = Vec::new();
    if matches!(demo, DemoKind::Demo022 | DemoKind::Demo028) {
        let aux_window_id = ctx.window_id + 1;
        let aux_binding = create_window(aux_window_id, &format!("{} Aux", demo.title()));
        setup_cmds.extend(aux_window_commands(
            aux_window_id,
            aux_binding.realm_id,
            ids,
        ));
        aux_windows.push(aux_window_id);
    }

    setup_cmds.extend(extra_setup_commands(demo, ctx, ui_realm_id, ids));
    let _ = send_commands(setup_cmds);
    let _ = receive_responses();

    let mut last_list_ms = 0_u64;
    run_loop_with_events(
        ctx.window_id,
        None,
        move |total_ms, delta_ms| {
            let mut cmds = hud.frame_commands(total_ms, delta_ms);
            cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Update(
                CmdModelUpdateArgs {
                    realm_id: ctx.realm_id,
                    model_id: ids.model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_rotation_y(total_ms as f32 * 0.0006)
                            * Mat4::from_rotation_x(total_ms as f32 * 0.0003),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                },
            )));

            if demo == DemoKind::Demo020 {
                cmds.push(EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
                    start: Vec3::new(-2.0, 0.0, 0.0),
                    end: Vec3::new(2.0, 0.0, 0.0),
                    color: Vec4::new(1.0, 0.2, 0.2, 1.0),
                }));
                cmds.push(EngineCmd::CmdGizmoDrawAabb(CmdGizmoDrawAabbArgs {
                    min: Vec3::new(-1.0, -1.0, -1.0),
                    max: Vec3::new(1.0, 1.0, 1.0),
                    color: Vec4::new(0.2, 1.0, 0.2, 0.5),
                }));
            }

            if demo == DemoKind::Demo027 && total_ms.saturating_sub(last_list_ms) >= 1000 {
                last_list_ms = total_ms;
                cmds.extend(list_commands(ctx.window_id));
            }

            if demo == DemoKind::Demo024 && total_ms.saturating_sub(last_list_ms) >= 1500 {
                last_list_ms = total_ms;
                println!("Demo 024 aguardando eventos de keyboard/mouse/touch/gamepad...");
            }

            cmds
        },
        move |event| {
            for aux_window in &aux_windows {
                if should_close_window(*aux_window, &event) {
                    let _ = send_commands(vec![EngineCmd::CmdWindowClose(
                        crate::core::window::CmdWindowCloseArgs {
                            window_id: *aux_window,
                        },
                    )]);
                }
            }
            false
        },
    )
}

fn base_scene_commands(ctx: DemoContext, ids: DemoIds) -> Vec<EngineCmd> {
    vec![
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
        create_camera_cmd(
            ctx.realm_id,
            ids.camera_id,
            "Main Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 2.2, 5.5), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(ctx.realm_id, ids.light_id, Vec4::new(2.0, 4.0, 2.0, 1.0)),
        create_ambient_light_cmd(
            ctx.realm_id,
            ids.light_id + 1,
            Vec4::new(0.2, 0.2, 0.2, 1.0),
            0.3,
        ),
        create_standard_material_cmd(
            ids.material_id,
            "Demo Material",
            Vec4::new(0.9, 0.5, 0.2, 1.0),
            None,
            None,
        ),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id: ctx.realm_id,
            model_id: ids.model_id,
            label: Some("Demo Cube Model".into()),
            geometry_id: ids.geometry_id,
            material_id: Some(ids.material_id),
            transform: Mat4::IDENTITY,
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: ctx.realm_id,
            target_id: ids.target_id,
            layout: TargetLayerLayout::default(),
            camera_id: Some(ids.camera_id),
            environment_id: None,
        }),
        create_shadow_config_cmd(ctx.window_id),
    ]
}

fn aux_window_commands(window_id: u32, realm_id: u32, ids: DemoIds) -> Vec<EngineCmd> {
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

fn extra_setup_commands(
    demo: DemoKind,
    ctx: DemoContext,
    ui_realm_id: u32,
    ids: DemoIds,
) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    match demo {
        DemoKind::Demo001 => {
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        clear_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        ..Default::default()
                    },
                }),
            ));
            cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
                realm_id: ctx.realm_id,
                target_id: ids.target_id,
                layout: TargetLayerLayout::default(),
                camera_id: Some(ids.camera_id),
                environment_id: Some(ids.env_id),
            }));
        }
        DemoKind::Demo002 => {
            cmds.push(EngineCmd::CmdWindowMeasurement(CmdWindowMeasurementArgs {
                window_id: ctx.window_id,
                get_position: true,
                get_size: true,
                get_outer_size: true,
                get_surface_size: true,
                ..Default::default()
            }));
            cmds.push(EngineCmd::CmdWindowState(CmdWindowStateArgs {
                window_id: ctx.window_id,
                title: Some("Vulfram Demo 002".into()),
                state: Some(EngineWindowState::Windowed),
                get_state: true,
                get_decorations: true,
                get_resizable: true,
                ..Default::default()
            }));
            cmds.push(EngineCmd::CmdWindowCursor(CmdWindowCursorArgs {
                window_id: ctx.window_id,
                visible: Some(true),
                mode: None,
                icon: Some(CursorIcon::Crosshair),
            }));
        }
        DemoKind::Demo003 => {
            cmds.push(EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
                kind: RealmKindDto::TwoD,
                output_surface_id: None,
                host_window_id: Some(ctx.window_id),
                importance: None,
                cache_policy: None,
                flags: None,
            }));
        }
        DemoKind::Demo004 => {
            let texture_target = ids.target_id + 1;
            cmds.push(EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
                target_id: texture_target,
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(UVec2::new(512, 512)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: Some(1),
            }));
            cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
                realm_id: ctx.realm_id,
                target_id: texture_target,
                layout: TargetLayerLayout::default(),
                camera_id: Some(ids.camera_id),
                environment_id: None,
            }));
            cmds.push(EngineCmd::CmdTargetLayerDispose(
                CmdTargetLayerDisposeArgs {
                    realm_id: ctx.realm_id,
                    target_id: texture_target,
                },
            ));
            cmds.push(EngineCmd::CmdTargetDispose(CmdTargetDisposeArgs {
                target_id: texture_target,
            }));
        }
        DemoKind::Demo005 => {
            cmds.push(EngineCmd::CmdCameraList(CmdCameraListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdCameraDispose(CmdCameraDisposeArgs {
                realm_id: ctx.realm_id,
                camera_id: ids.camera_id + 99,
            }));
        }
        DemoKind::Demo006 => {
            let mut bones = vec![Mat4::IDENTITY; 16];
            for (idx, bone) in bones.iter_mut().enumerate() {
                *bone = Mat4::from_translation(Vec3::new(0.0, idx as f32 * 0.01, 0.0));
            }
            upload_buffer(
                70_000 + ids.model_id as u64,
                crate::core::buffers::state::UploadType::Raw,
                &bones,
            );
            cmds.push(EngineCmd::CmdPoseUpdate(CmdPoseUpdateArgs {
                realm_id: ctx.realm_id,
                model_id: ids.model_id,
                bone_count: 16,
                matrices_buffer_id: 70_000 + ids.model_id as u64,
                window_id: Some(ctx.window_id),
            }));
            cmds.push(EngineCmd::CmdModelList(CmdModelListArgs {
                window_id: ctx.window_id,
            }));
        }
        DemoKind::Demo007 => {
            let _ = build_skinned_plane(4, 4, 1.0, 4);
            let positions: [Vec3; 3] = [
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ];
            let normals: [Vec3; 3] = [Vec3::Z, Vec3::Z, Vec3::Z];
            let uvs: [Vec2; 3] = [
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.5, 0.0),
            ];
            let indices: [u32; 3] = [0, 1, 2];

            let base_buffer = 80_000 + ids.geometry_id as u64;
            upload_buffer(
                base_buffer,
                crate::core::buffers::state::UploadType::VertexData,
                &positions,
            );
            upload_buffer(
                base_buffer + 1,
                crate::core::buffers::state::UploadType::VertexData,
                &normals,
            );
            upload_buffer(
                base_buffer + 2,
                crate::core::buffers::state::UploadType::VertexData,
                &uvs,
            );
            upload_buffer(
                base_buffer + 3,
                crate::core::buffers::state::UploadType::IndexData,
                &indices,
            );

            cmds.push(EngineCmd::CmdGeometryUpsert(CmdGeometryUpsertArgs::Create(
                CmdGeometryCreateArgs {
                    geometry_id: ids.geometry_id + 200,
                    label: Some("Triangle Custom".into()),
                    entries: vec![
                        GeometryPrimitiveEntry {
                            primitive_type: crate::core::resources::GeometryPrimitiveType::Position,
                            buffer_id: base_buffer,
                        },
                        GeometryPrimitiveEntry {
                            primitive_type: crate::core::resources::GeometryPrimitiveType::Normal,
                            buffer_id: base_buffer + 1,
                        },
                        GeometryPrimitiveEntry {
                            primitive_type: crate::core::resources::GeometryPrimitiveType::UV,
                            buffer_id: base_buffer + 2,
                        },
                        GeometryPrimitiveEntry {
                            primitive_type: crate::core::resources::GeometryPrimitiveType::Index,
                            buffer_id: base_buffer + 3,
                        },
                    ],
                },
            )));
            cmds.push(EngineCmd::CmdGeometryList(CmdGeometryListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdGeometryDispose(CmdGeometryDisposeArgs {
                geometry_id: ids.geometry_id + 200,
            }));
        }
        DemoKind::Demo008 => {
            cmds.push(EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(
                CmdMaterialCreateArgs {
                    material_id: ids.material_id + 100,
                    label: Some("PBR Material".into()),
                    kind: MaterialKind::Pbr,
                    options: Some(MaterialOptions::Pbr(PbrOptions {
                        base_color: Vec4::new(0.7, 0.8, 1.0, 1.0),
                        metallic: 0.8,
                        roughness: 0.2,
                        ..Default::default()
                    })),
                },
            )));
            cmds.push(EngineCmd::CmdMaterialList(CmdMaterialListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdMaterialDispose(CmdMaterialDisposeArgs {
                material_id: ids.material_id + 100,
            }));
        }
        DemoKind::Demo009 => {
            cmds.push(EngineCmd::CmdLightUpsert(CmdLightUpsertArgs::Create(
                CmdLightCreateArgs {
                    realm_id: ctx.realm_id,
                    light_id: ids.light_id + 100,
                    label: Some("Extra Light".into()),
                    kind: Some(LightKind::Spot),
                    position: Some(Vec4::new(0.0, 4.0, 4.0, 1.0)),
                    direction: Some(Vec4::new(0.0, -1.0, -1.0, 0.0)),
                    color: Some(Vec4::new(0.7, 0.8, 1.0, 1.0)),
                    ground_color: None,
                    intensity: Some(10.0),
                    range: Some(16.0),
                    spot_inner_outer: Some(Vec2::new(0.4, 0.8)),
                    layer_mask: 0xFFFF_FFFF,
                    cast_shadow: true,
                },
            )));
            cmds.push(EngineCmd::CmdLightList(CmdLightListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdLightDispose(CmdLightDisposeArgs {
                realm_id: ctx.realm_id,
                light_id: ids.light_id + 100,
            }));
        }
        DemoKind::Demo010 => {
            cmds.push(EngineCmd::CmdTextureCreateSolidColor(
                CmdTextureCreateSolidColorArgs {
                    texture_id: ids.texture_id,
                    label: Some("Solid Texture".into()),
                    color: Vec4::new(0.1, 0.6, 0.9, 1.0),
                    srgb: Some(true),
                    mode: TextureCreateMode::Standalone,
                    atlas_options: None,
                },
            ));
            cmds.push(EngineCmd::CmdTextureBindTarget(CmdTextureBindTargetArgs {
                texture_id: ids.texture_id,
                target_id: ids.target_id,
                label: Some("Main target bind".into()),
            }));
            cmds.push(EngineCmd::CmdTextureList(CmdTextureListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdTextureDispose(CmdTextureDisposeArgs {
                texture_id: ids.texture_id,
            }));
        }
        DemoKind::Demo011 => {
            let image_bytes = load_texture_bytes("assets/colo_test_texture.png");
            upload_texture_bytes(&image_bytes, 81_000 + ids.texture_id as u64);
            cmds.push(EngineCmd::CmdTextureCreateFromBuffer(
                CmdTextureCreateFromBufferArgs {
                    texture_id: ids.texture_id + 1,
                    label: Some("Async texture".into()),
                    buffer_id: 81_000 + ids.texture_id as u64,
                    srgb: Some(true),
                    mode: TextureCreateMode::Standalone,
                    atlas_options: None,
                },
            ));
            cmds.push(EngineCmd::CmdUploadBufferDiscardAll(
                CmdUploadBufferDiscardAllArgs {},
            ));
        }
        DemoKind::Demo012 => {
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        msaa: MsaaConfig {
                            enabled: true,
                            sample_count: 4,
                        },
                        skybox: SkyboxConfig {
                            mode: SkyboxMode::Procedural,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
            ));
            cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
                realm_id: ctx.realm_id,
                target_id: ids.target_id,
                layout: TargetLayerLayout::default(),
                camera_id: Some(ids.camera_id),
                environment_id: Some(ids.env_id),
            }));
            cmds.push(EngineCmd::CmdEnvironmentDispose(
                CmdEnvironmentDisposeArgs {
                    environment_id: ids.env_id,
                },
            ));
        }
        DemoKind::Demo013 => {
            cmds.push(EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
                window_id: ctx.window_id,
                config: ShadowConfig::default(),
            }));
        }
        DemoKind::Demo014 => {
            let mut post = PostProcessConfig {
                filter_enabled: true,
                filter_exposure: 1.1,
                filter_gamma: 2.2,
                filter_saturation: 1.15,
                filter_contrast: 1.1,
                filter_vignette: 0.1,
                filter_grain: 0.03,
                filter_chromatic_aberration: 0.08,
                filter_blur: 0.0,
                filter_sharpen: 0.12,
                filter_tonemap_mode: 2,
                outline_enabled: true,
                outline_strength: 0.75,
                outline_threshold: 0.99,
                outline_width: 1.0,
                outline_quality: 1.0,
                filter_posterize_steps: 0.0,
                cell_shading: false,
                ssao_enabled: false,
                ssao_strength: 1.0,
                ssao_radius: 0.75,
                ssao_bias: 0.025,
                ssao_power: 1.5,
                ssao_blur_radius: 2.0,
                ssao_blur_depth_threshold: 0.02,
                bloom_enabled: false,
                bloom_threshold: 1.0,
                bloom_knee: 0.5,
                bloom_intensity: 0.8,
                bloom_scatter: 0.7,
            };
            post.outline_threshold = post.outline_threshold.clamp(0.0, 0.99);
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        post,
                        ..Default::default()
                    },
                }),
            ));
        }
        DemoKind::Demo015 => {
            let post = PostProcessConfig {
                ssao_enabled: true,
                ssao_strength: 1.3,
                ssao_radius: 0.9,
                ssao_bias: 0.03,
                ssao_power: 1.6,
                ssao_blur_radius: 2.5,
                ..Default::default()
            };
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        post,
                        ..Default::default()
                    },
                }),
            ));
        }
        DemoKind::Demo016 => {
            let post = PostProcessConfig {
                bloom_enabled: true,
                bloom_threshold: 0.9,
                bloom_knee: 0.6,
                bloom_intensity: 0.9,
                bloom_scatter: 0.7,
                ..Default::default()
            };
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        post,
                        ..Default::default()
                    },
                }),
            ));
        }
        DemoKind::Demo017 => {
            cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
                document_id: ids.ui_doc_extra,
                realm_id: ui_realm_id,
                rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
                theme_id: None,
            }));
            cmds.push(EngineCmd::CmdUiApplyOps(
                crate::core::ui::cmd::CmdUiApplyOpsArgs {
                    document_id: ids.ui_doc_extra,
                    version: 1,
                    ops: vec![UiOp::Add {
                        parent: None,
                        node: UiNode {
                            id: ids.ui_node_extra,
                            kind: UiNodeKind::Text,
                            props: UiNodeProps::Text {
                                text: "Demo 017 UI Runtime".into(),
                                size: Some(20.0),
                                color: None,
                            },
                            tooltip: None,
                            context_menu: None,
                            anim: None,
                            display: None,
                            visible: None,
                            opacity: None,
                            z_index: Some(1000),
                        },
                        index: None,
                    }],
                },
            ));
        }
        DemoKind::Demo018 => {
            cmds.push(EngineCmd::CmdUiDocumentGetTree(CmdUiDocumentGetTreeArgs {
                document_id: 90_000 + demo.number() * 16,
            }));
            cmds.push(EngineCmd::CmdUiDocumentGetLayoutRects(
                CmdUiDocumentGetLayoutRectsArgs {
                    document_id: 90_000 + demo.number() * 16,
                },
            ));
            cmds.push(EngineCmd::CmdUiFocusSet(CmdUiFocusSetArgs {
                window_id: ctx.window_id,
                realm_id: ui_realm_id,
                document_id: 90_000 + demo.number() * 16,
                node_id: None,
            }));
            cmds.push(EngineCmd::CmdUiFocusGet(CmdUiFocusGetArgs {
                window_id: Some(ctx.window_id),
            }));
            cmds.push(EngineCmd::CmdUiEventTraceSet(CmdUiEventTraceSetArgs {
                level: Some(crate::core::input::events::PointerTraceLevel::Full),
                sampling_percent: Some(100),
            }));
            cmds.push(EngineCmd::CmdUiDebugSet(CmdUiDebugSetArgs {
                enabled: true,
                show_bounds: true,
                show_ids: true,
                show_profile: true,
            }));
        }
        DemoKind::Demo019 => {
            let image_bytes = load_texture_bytes("assets/alpha_test_texture.png");
            upload_texture_bytes(&image_bytes, 82_000 + ids.texture_id as u64);
            cmds.push(EngineCmd::CmdUiImageCreateFromBuffer(
                CmdUiImageCreateFromBufferArgs {
                    image_id: ids.aux_id,
                    buffer_id: 82_000 + ids.texture_id as u64,
                    label: Some("UI Image".into()),
                },
            ));
            cmds.push(EngineCmd::CmdUiClipboardPaste(CmdUiClipboardPasteArgs {
                window_id: ctx.window_id,
                text: "Clipboard from demo 019".into(),
            }));
            cmds.push(EngineCmd::CmdUiScreenshotReply(CmdUiScreenshotReplyArgs {
                window_id: ctx.window_id,
                realm_id: Some(ui_realm_id),
                width: 1,
                height: 1,
                rgba: vec![255, 255, 255, 255],
            }));
            cmds.push(EngineCmd::CmdUiAccessKitActionRequest(
                CmdUiAccessKitActionRequestArgs {
                    window_id: ctx.window_id,
                    realm_id: Some(ui_realm_id),
                    action: "focus-next".into(),
                },
            ));
            cmds.push(EngineCmd::CmdUiImageDispose(CmdUiImageDisposeArgs {
                image_id: ids.aux_id,
            }));
        }
        DemoKind::Demo020 => {}
        DemoKind::Demo021 => {
            let realm_plane_target = ids.target_id + 700;
            cmds.push(EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
                target_id: realm_plane_target,
                kind: TargetKind::RealmPlane,
                window_id: None,
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            }));
            cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
                realm_id: ctx.realm_id,
                target_id: realm_plane_target,
                layout: TargetLayerLayout {
                    left: DimensionValue::Px(20.0),
                    top: DimensionValue::Px(20.0),
                    width: DimensionValue::Percent(60.0),
                    height: DimensionValue::Percent(60.0),
                    z_index: 5,
                    blend_mode: 0,
                    clip: None,
                },
                camera_id: Some(ids.camera_id),
                environment_id: None,
            }));
            cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
                document_id: ids.ui_doc_extra,
                realm_id: ui_realm_id,
                rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
                theme_id: None,
            }));
            cmds.push(EngineCmd::CmdUiApplyOps(
                crate::core::ui::cmd::CmdUiApplyOpsArgs {
                    document_id: ids.ui_doc_extra,
                    version: 1,
                    ops: vec![UiOp::Add {
                        parent: None,
                        node: UiNode {
                            id: ids.ui_node_extra,
                            kind: UiNodeKind::WidgetRealmViewport,
                            props: UiNodeProps::WidgetRealmViewport {
                                target_id: realm_plane_target,
                                size: None,
                            },
                            tooltip: None,
                            context_menu: None,
                            anim: None,
                            display: None,
                            visible: None,
                            opacity: None,
                            z_index: Some(200),
                        },
                        index: None,
                    }],
                },
            ));
        }
        DemoKind::Demo022 => {}
        DemoKind::Demo023 => {
            let audio_bytes = std::fs::read("assets/audio.wav").unwrap_or_default();
            upload_binary_bytes(&audio_bytes, 83_000 + ids.aux_id as u64);
            cmds.push(EngineCmd::CmdAudioResourceUpsert(
                CmdAudioResourceUpsertArgs {
                    resource_id: ids.aux_id,
                    buffer_id: 83_000 + ids.aux_id as u64,
                    total_bytes: Some(audio_bytes.len() as u64),
                    offset_bytes: Some(0),
                },
            ));
            cmds.push(EngineCmd::CmdAudioListenerUpsert(
                crate::core::cmd::CmdAudioListenerUpsertArgs::Create(
                    crate::core::audio::cmd::CmdAudioListenerCreateArgs {
                        realm_id: ctx.realm_id,
                        model_id: ids.model_id,
                    },
                ),
            ));
            cmds.push(EngineCmd::CmdAudioSourceUpsert(
                crate::core::cmd::CmdAudioSourceUpsertArgs::Create(CmdAudioSourceCreateArgs {
                    realm_id: ctx.realm_id,
                    source_id: ids.aux_id + 1,
                    model_id: ids.model_id,
                    position: Vec3::ZERO,
                    velocity: Vec3::ZERO,
                    orientation: glam::Quat::IDENTITY,
                    gain: 1.0,
                    pitch: 1.0,
                    spatial: AudioSpatialParamsDto::default(),
                }),
            ));
            cmds.push(EngineCmd::CmdAudioSourceTransport(
                CmdAudioSourceTransportArgs {
                    source_id: ids.aux_id + 1,
                    action: AudioSourceTransportActionDto::Play,
                    resource_id: Some(ids.aux_id),
                    timeline_id: None,
                    intensity: Some(1.0),
                    delay_ms: Some(0),
                    mode: Some(AudioPlayModeDto::Loop),
                },
            ));
            cmds.push(EngineCmd::CmdAudioStateGet(
                crate::core::audio::cmd::CmdAudioStateGetArgs::default(),
            ));
        }
        DemoKind::Demo024 => {}
        DemoKind::Demo025 => {
            cmds.push(EngineCmd::CmdSystemDiagnosticsSet(
                CmdSystemDiagnosticsSetArgs {
                    profiling_enabled: Some(true),
                    profiling_detail: Some(ProfilingDetailLevel::Basic),
                    profiling_sampling_percent: Some(100),
                    profiling_window_frames: Some(60),
                    trace_level: Some(crate::core::input::events::PointerTraceLevel::Errors),
                    trace_sampling_percent: Some(100),
                },
            ));
            cmds.push(EngineCmd::CmdNotificationSend(CmdNotificationSendArgs {
                id: Some("demo025-start".into()),
                title: "Demo 025".into(),
                body: "Diagnostics and notifications active".into(),
                level: NotificationLevel::Info,
                timeout: Some(1500),
            }));
        }
        DemoKind::Demo026 => {
            let post = PostProcessConfig {
                filter_enabled: true,
                outline_enabled: true,
                outline_strength: 0.7,
                outline_threshold: 0.2,
                ssao_enabled: true,
                bloom_enabled: true,
                bloom_intensity: 1.0,
                ..Default::default()
            };
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        msaa: MsaaConfig {
                            enabled: true,
                            sample_count: 4,
                        },
                        skybox: SkyboxConfig {
                            mode: SkyboxMode::Procedural,
                            ..Default::default()
                        },
                        post,
                        ..Default::default()
                    },
                }),
            ));
            cmds.push(EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
                realm_id: ctx.realm_id,
                target_id: ids.target_id,
                layout: TargetLayerLayout::default(),
                camera_id: Some(ids.camera_id),
                environment_id: Some(ids.env_id),
            }));
        }
        DemoKind::Demo027 => {
            cmds.extend(list_commands(ctx.window_id));
        }
        DemoKind::Demo028 => {
            cmds.extend(list_commands(ctx.window_id));
            cmds.push(EngineCmd::CmdNotificationSend(CmdNotificationSendArgs {
                id: Some("demo028-start".into()),
                title: "Demo 028".into(),
                body: "Full engine integration".into(),
                level: NotificationLevel::Success,
                timeout: Some(1200),
            }));
        }
    }

    cmds
}

fn should_close_window(window_id: u32, event: &EngineEvent) -> bool {
    match event {
        EngineEvent::Window(crate::core::window::WindowEvent::OnCloseRequest { window_id: id }) => {
            *id == window_id
        }
        EngineEvent::Keyboard(KeyboardEvent::OnInput {
            window_id: id,
            key_code,
            state: ElementState::Pressed,
            modifiers,
            ..
        }) if *id == window_id => *key_code == 106 || (*key_code == 41 && modifiers.ctrl),
        _ => false,
    }
}

fn create_ui_realm(window_id: u32) -> Option<u32> {
    let _ = send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        output_surface_id: None,
        host_window_id: Some(window_id),
        importance: None,
        cache_policy: None,
        flags: None,
    })]);

    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
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

fn create_and_dispose_temp_realm(window_id: u32) -> bool {
    let _ = send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        output_surface_id: None,
        host_window_id: Some(window_id),
        importance: None,
        cache_policy: None,
        flags: None,
    })]);

    let mut realm_id: Option<u32> = None;
    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
            if let CommandResponse::RealmCreate(result) = envelope.response
                && result.success
            {
                realm_id = result.realm_id;
                break;
            }
        }
        if realm_id.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }

    if let Some(temp_realm_id) = realm_id {
        let _ = send_commands(vec![EngineCmd::CmdRealmDispose(CmdRealmDisposeArgs {
            realm_id: temp_realm_id,
        })]);
        return true;
    }

    false
}

fn list_commands(window_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdModelList(CmdModelListArgs { window_id }),
        EngineCmd::CmdMaterialList(CmdMaterialListArgs { window_id }),
        EngineCmd::CmdTextureList(CmdTextureListArgs { window_id }),
        EngineCmd::CmdGeometryList(CmdGeometryListArgs { window_id }),
        EngineCmd::CmdLightList(CmdLightListArgs { window_id }),
        EngineCmd::CmdCameraList(CmdCameraListArgs { window_id }),
    ]
}
