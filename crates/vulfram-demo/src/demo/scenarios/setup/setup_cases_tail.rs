use super::*;

pub(super) fn append_setup_commands_tail(
    scenario: u32,
    ctx: DemoContext,
    ui_realm_id: u32,
    ids: DemoIds,
    cmds: &mut Vec<EngineCmd>,
) {
    match scenario {
        16 => {
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
        17 => {
            cmds.push(EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
                document_id: ids.ui_doc_extra,
                realm_id: ui_realm_id,
                rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
                theme_id: None,
            }));
            cmds.push(EngineCmd::CmdUiApplyOps(
                vulfram_core::core::ui::cmd::CmdUiApplyOpsArgs {
                    document_id: ids.ui_doc_extra,
                    version: 1,
                    ops: vec![
                        UiOp::Add {
                            parent: None,
                            node: UiNode {
                                id: ids.ui_node_extra,
                                kind: UiNodeKind::Area,
                                props: UiNodeProps::Area {
                                    label: Some("suite-d-runtime-anchor".into()),
                                    x: Some(8.0),
                                    y: Some(660.0),
                                    draggable: Some(false),
                                    size: None,
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
                        },
                        UiOp::Add {
                            parent: Some(ids.ui_node_extra),
                            node: UiNode {
                                id: ids.ui_node_extra + 1,
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
                                z_index: Some(1001),
                            },
                            index: None,
                        },
                    ],
                },
            ));
        }
        18 => {
            cmds.push(EngineCmd::CmdUiDocumentGetTree(CmdUiDocumentGetTreeArgs {
                document_id: 90_000 + scenario * 16,
            }));
            cmds.push(EngineCmd::CmdUiDocumentGetLayoutRects(
                CmdUiDocumentGetLayoutRectsArgs {
                    document_id: 90_000 + scenario * 16,
                },
            ));
            cmds.push(EngineCmd::CmdUiFocusSet(CmdUiFocusSetArgs {
                window_id: ctx.window_id,
                realm_id: ui_realm_id,
                document_id: 90_000 + scenario * 16,
                node_id: None,
            }));
            cmds.push(EngineCmd::CmdUiFocusGet(CmdUiFocusGetArgs {
                window_id: Some(ctx.window_id),
            }));
            cmds.push(EngineCmd::CmdUiEventTraceSet(CmdUiEventTraceSetArgs {
                level: Some(vulfram_core::core::input::events::PointerTraceLevel::Full),
                sampling_percent: Some(100),
            }));
            cmds.push(EngineCmd::CmdUiDebugSet(CmdUiDebugSetArgs {
                enabled: true,
                show_bounds: true,
                show_ids: true,
                show_profile: true,
            }));
        }
        19 => {
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
        20 => {
            let sun_a_id = ids.light_id + 200;
            let sun_b_id = ids.light_id + 201;
            cmds.push(EngineCmd::CmdModelDispose(CmdModelDisposeArgs {
                realm_id: ctx.realm_id,
                model_id: ids.ground_model_id,
            }));
            cmds.push(EngineCmd::CmdLightDispose(CmdLightDisposeArgs {
                realm_id: ctx.realm_id,
                light_id: ids.light_id,
            }));
            cmds.push(EngineCmd::CmdLightDispose(CmdLightDisposeArgs {
                realm_id: ctx.realm_id,
                light_id: ids.light_id + 1,
            }));
            cmds.push(EngineCmd::CmdLightUpsert(CmdLightUpsertArgs::Create(
                CmdLightCreateArgs {
                    realm_id: ctx.realm_id,
                    light_id: sun_a_id,
                    label: Some("Demo 4 Sun A".into()),
                    kind: Some(LightKind::Directional),
                    position: Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                    direction: Some(Vec4::new(-0.45, -0.85, -0.30, 0.0)),
                    color: Some(Vec4::new(1.0, 0.92, 0.75, 1.0)),
                    ground_color: None,
                    intensity: Some(0.5),
                    range: Some(0.0),
                    spot_inner_outer: None,
                    layer_mask: 0xFFFF_FFFF,
                    cast_shadow: false,
                },
            )));
            cmds.push(EngineCmd::CmdLightUpsert(CmdLightUpsertArgs::Create(
                CmdLightCreateArgs {
                    realm_id: ctx.realm_id,
                    light_id: sun_b_id,
                    label: Some("Demo 4 Sun B".into()),
                    kind: Some(LightKind::Directional),
                    position: Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                    direction: Some(Vec4::new(0.55, -0.65, 0.48, 0.0)),
                    color: Some(Vec4::new(0.70, 0.82, 1.0, 1.0)),
                    ground_color: None,
                    intensity: Some(0.5),
                    range: Some(0.0),
                    spot_inner_outer: None,
                    layer_mask: 0xFFFF_FFFF,
                    cast_shadow: false,
                },
            )));
            cmds.push(EngineCmd::CmdEnvironmentUpsert(
                CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                    environment_id: ids.env_id,
                    config: EnvironmentConfig {
                        skybox: SkyboxConfig {
                            mode: SkyboxMode::Procedural,
                            ground_color: Vec3::new(0.34, 0.22, 0.14),
                            horizon_color: Vec3::new(0.98, 0.86, 0.38),
                            sky_color: Vec3::new(0.24, 0.52, 0.92),
                            horizon_ground_threshold: 0.98,
                            horizon_sky_threshold: 0.98,
                            directional_lights: vec![
                                SkyboxDirectionalLightSun {
                                    light_id: sun_a_id,
                                    solid_size: 0.002,
                                    gradient_size: 0.012,
                                },
                                SkyboxDirectionalLightSun {
                                    light_id: sun_b_id,
                                    solid_size: 0.0016,
                                    gradient_size: 0.010,
                                },
                            ],
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
        21 => {
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
                vulfram_core::core::ui::cmd::CmdUiApplyOpsArgs {
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
        22 => {}
        23 => {
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
                vulfram_core::core::cmd::CmdAudioListenerUpsertArgs::Create(
                    vulfram_core::core::audio::cmd::CmdAudioListenerCreateArgs {
                        realm_id: ctx.realm_id,
                        model_id: ids.model_id,
                    },
                ),
            ));
            cmds.push(EngineCmd::CmdAudioSourceUpsert(
                vulfram_core::core::cmd::CmdAudioSourceUpsertArgs::Create(
                    CmdAudioSourceCreateArgs {
                        realm_id: ctx.realm_id,
                        source_id: ids.aux_id + 1,
                        model_id: ids.model_id,
                        position: Vec3::ZERO,
                        velocity: Vec3::ZERO,
                        orientation: glam::Quat::IDENTITY,
                        gain: 1.0,
                        pitch: 1.0,
                        spatial: AudioSpatialParamsDto::default(),
                    },
                ),
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
                vulfram_core::core::audio::cmd::CmdAudioStateGetArgs::default(),
            ));
        }
        24 => {}
        25 => {
            cmds.push(EngineCmd::CmdSystemDiagnosticsSet(
                CmdSystemDiagnosticsSetArgs {
                    profiling_enabled: Some(true),
                    profiling_detail: Some(ProfilingDetailLevel::Basic),
                    profiling_sampling_percent: Some(100),
                    profiling_window_frames: Some(60),
                    trace_level: Some(vulfram_core::core::input::events::PointerTraceLevel::Errors),
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
        26 => {
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
        27 => {
            cmds.extend(list_commands(ctx.window_id));
        }
        28 => {
            cmds.extend(list_commands(ctx.window_id));
            cmds.push(EngineCmd::CmdNotificationSend(CmdNotificationSendArgs {
                id: Some("demo028-start".into()),
                title: "Demo 028".into(),
                body: "Full engine integration".into(),
                level: NotificationLevel::Success,
                timeout: Some(1200),
            }));
        }
        _ => {}
    }
}
