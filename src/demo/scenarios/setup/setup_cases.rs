use super::*;
mod setup_cases_tail;
pub(super) fn extra_setup_commands(
    scenario: u32,
    ctx: DemoContext,
    ui_realm_id: u32,
    ids: DemoIds,
) -> Vec<EngineCmd> {
    let mut cmds = Vec::new();

    match scenario {
        1 => {}
        2 => {
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
        3 => {
            cmds.push(EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
                kind: RealmKindDto::TwoD,
                importance: None,
                cache_policy: None,
                flags: None,
            }));
        }
        4 => {
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
        5 => {
            cmds.push(EngineCmd::CmdCameraList(CmdCameraListArgs {
                window_id: ctx.window_id,
            }));
            cmds.push(EngineCmd::CmdCameraDispose(CmdCameraDisposeArgs {
                realm_id: ctx.realm_id,
                camera_id: ids.camera_id + 99,
            }));
        }
        6 => {
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
        7 => {
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
        8 => {
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
        9 => {
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
        10 => {
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
        11 => {
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
        12 => {
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
        }
        13 => {
            cmds.push(EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
                window_id: ctx.window_id,
                config: ShadowConfig::default(),
            }));
        }
        14 => {
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
        15 => {
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
        _ => {
            setup_cases_tail::append_setup_commands_tail(
                scenario,
                ctx,
                ui_realm_id,
                ids,
                &mut cmds,
            );
        }
    }

    cmds
}
