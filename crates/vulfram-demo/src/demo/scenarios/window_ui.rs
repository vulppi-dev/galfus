use super::*;

pub(super) fn run_demo_2_window_ui(ctx: DemoContext) -> bool {
    let demo_number = 2;
    let ids = DemoIds::from_number(demo_number);
    let Some(ui_realm_id) = create_ui_realm(ctx.window_id) else {
        return false;
    };
    let scene_camera_id = ids.camera_id + 200;
    let scene_light_id = ids.light_id + 200;
    let scene_env_id = ids.env_id + 200;
    let scene_cube_geometry_id = ids.geometry_id + 200;
    let scene_cube_material_id = ids.material_id + 200;
    let scene_cube_model_id = ids.model_id + 200;
    let scene_ground_geometry_id = ids.ground_geometry_id + 200;
    let scene_ground_material_id = ids.ground_material_id + 200;
    let scene_ground_model_id = ids.ground_model_id + 200;

    let target_id = ids.target_id + 2_000;
    let control_doc_id = ids.ui_doc_extra + 200;
    let control_root_id = ids.ui_node_extra + 200;
    let measurement_text_id = control_root_id + 1;
    let btn_measure = control_root_id + 10;
    let btn_windowed = control_root_id + 11;
    let btn_maximized = control_root_id + 12;
    let btn_minimized = control_root_id + 13;
    let btn_fullscreen = control_root_id + 14;
    let btn_borderless = control_root_id + 15;
    let btn_cursor_default = control_root_id + 16;
    let btn_cursor_crosshair = control_root_id + 17;

    let mut hud = FpsHud::new(demo_number);
    let mut setup_cmds = vec![
        EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
            target_id,
            kind: TargetKind::Window,
            window_id: Some(ctx.window_id),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        }),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: scene_env_id,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.03, 0.06, 0.10, 1.0),
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::None,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: scene_cube_geometry_id,
            label: Some("Demo2 Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: scene_ground_geometry_id,
            label: Some("Demo2 Ground Plane".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        create_camera_cmd(
            ctx.realm_id,
            scene_camera_id,
            "Demo2 Scene Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 2.4, 5.8), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(
            ctx.realm_id,
            scene_light_id,
            Vec4::new(2.4, 4.2, 2.3, 1.0),
            12.0,
        ),
        create_ambient_light_cmd(
            ctx.realm_id,
            scene_light_id + 1,
            Vec4::new(0.23, 0.27, 0.32, 1.0),
            0.12,
        ),
        create_standard_material_cmd(
            scene_cube_material_id,
            "Demo2 Cube Material",
            Vec4::new(0.92, 0.42, 0.15, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            scene_ground_material_id,
            "Demo2 Ground Material",
            Vec4::new(0.18, 0.22, 0.28, 1.0),
            None,
            None,
        ),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id: ctx.realm_id,
            model_id: scene_cube_model_id,
            label: Some("Demo2 Cube Model".into()),
            geometry_id: scene_cube_geometry_id,
            material_id: Some(scene_cube_material_id),
            transform: Mat4::IDENTITY,
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Create(CmdModelCreateArgs {
            realm_id: ctx.realm_id,
            model_id: scene_ground_model_id,
            label: Some("Demo2 Ground Model".into()),
            geometry_id: scene_ground_geometry_id,
            material_id: Some(scene_ground_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.2, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::splat(18.0)),
            layer_mask: 0xFFFF_FFFF,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: ctx.realm_id,
            target_id,
            layout: TargetLayerLayout::default(),
            camera_id: Some(scene_camera_id),
            environment_id: Some(scene_env_id),
        }),
        create_shadow_config_cmd(ctx.window_id),
        EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
            realm_id: ui_realm_id,
            target_id,
            layout: TargetLayerLayout::default(),
            camera_id: None,
            environment_id: None,
        }),
        EngineCmd::CmdUiDocumentCreate(CmdUiDocumentCreateArgs {
            document_id: control_doc_id,
            realm_id: ui_realm_id,
            rect: glam::vec4(0.0, 0.0, 0.0, 0.0),
            theme_id: None,
        }),
    ];
    setup_cmds.extend(hud.setup_commands(ui_realm_id));
    setup_cmds.push(EngineCmd::CmdUiApplyOps(
        vulfram_core::core::ui::cmd::CmdUiApplyOpsArgs {
            document_id: control_doc_id,
            version: 1,
            ops: vec![
                UiOp::Add {
                    parent: None,
                    node: UiNode {
                        id: control_root_id,
                        kind: UiNodeKind::Container,
                        props: UiNodeProps::Container {
                            layout: Default::default(),
                            padding: None,
                            size: None,
                            scroll_x: false,
                            scroll_y: false,
                        },
                        tooltip: None,
                        context_menu: None,
                        anim: None,
                        display: None,
                        visible: None,
                        opacity: None,
                        z_index: Some(100),
                    },
                    index: None,
                },
                UiOp::Add {
                    parent: Some(control_root_id),
                    node: UiNode {
                        id: measurement_text_id,
                        kind: UiNodeKind::Text,
                        props: UiNodeProps::Text {
                            text: "Measurement: aguardando...".into(),
                            size: Some(18.0),
                            color: None,
                        },
                        tooltip: None,
                        context_menu: None,
                        anim: None,
                        display: None,
                        visible: None,
                        opacity: None,
                        z_index: Some(101),
                    },
                    index: None,
                },
                ui_button_op(control_root_id, btn_measure, "Measure"),
                ui_button_op(control_root_id, btn_windowed, "State: Windowed"),
                ui_button_op(control_root_id, btn_maximized, "State: Maximized"),
                ui_button_op(control_root_id, btn_minimized, "State: Minimized"),
                ui_button_op(control_root_id, btn_fullscreen, "State: Fullscreen"),
                ui_button_op(
                    control_root_id,
                    btn_borderless,
                    "State: Windowed Fullscreen",
                ),
                ui_button_op(control_root_id, btn_cursor_default, "Cursor: Default"),
                ui_button_op(control_root_id, btn_cursor_crosshair, "Cursor: Crosshair"),
            ],
        },
    ));
    setup_cmds.push(window_measurement_cmd(ctx.window_id));
    let _ = send_commands(setup_cmds);
    let _ = receive_responses();

    let mut control_version: u64 = 1;
    let mut last_frame_time = std::time::Instant::now();
    let mut total_ms: u64 = 0;
    let mut last_measurement_text = String::new();
    let mut measured_position = String::from("-");
    let mut measured_size = String::from("-");
    let mut measured_outer_size = String::from("-");
    let target_frame_time = Duration::from_millis(16);

    loop {
        let now = std::time::Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms = total_ms.saturating_add(delta_ms as u64);

        let mut frame_cmds = hud.frame_commands(total_ms, delta_ms);
        frame_cmds.push(EngineCmd::CmdModelUpsert(CmdModelUpsertArgs::Update(
            CmdModelUpdateArgs {
                realm_id: ctx.realm_id,
                model_id: scene_cube_model_id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_rotation_y(total_ms as f32 * 0.0007)
                        * Mat4::from_rotation_x(total_ms as f32 * 0.00035),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            },
        )));
        frame_cmds.push(window_measurement_cmd(ctx.window_id));
        if !frame_cmds.is_empty() {
            let _ = send_commands(frame_cmds);
        }

        if core::vulfram_tick(total_ms, delta_ms) != vulfram_core::core::VulframResult::Success {
            return false;
        }

        let mut latest_measurement_text: Option<String> = None;
        for envelope in receive_responses() {
            if let CommandResponse::WindowMeasurement(result) = envelope.response
                && result.success
            {
                if result.position.is_none() && result.size.is_none() && result.outer_size.is_none()
                {
                    continue;
                }

                if let Some(position) = result.position {
                    measured_position = format!("({}, {})", position.x, position.y);
                }
                if let Some(size) = result.size {
                    measured_size = format!("{}x{}", size.x, size.y);
                }
                if let Some(outer_size) = result.outer_size {
                    measured_outer_size = format!("{}x{}", outer_size.x, outer_size.y);
                }
                let text = format!(
                    "Measurement: pos={} | size={} | outer={}",
                    measured_position, measured_size, measured_outer_size
                );
                latest_measurement_text = Some(text);
            }
        }
        if let Some(text) = latest_measurement_text
            && text != last_measurement_text
        {
            last_measurement_text = text.clone();
            control_version = control_version.saturating_add(1);
            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                vulfram_core::core::ui::cmd::CmdUiApplyOpsArgs {
                    document_id: control_doc_id,
                    version: control_version,
                    ops: vec![UiOp::Set {
                        node_id: measurement_text_id,
                        props: UiNodeProps::Text {
                            text,
                            size: Some(18.0),
                            color: None,
                        },
                    }],
                },
            )]);
        }

        let events = receive_events();
        for event in events {
            if should_close_window(ctx.window_id, &event) {
                let _ = send_commands(vec![EngineCmd::CmdWindowClose(
                    vulfram_core::core::window::CmdWindowCloseArgs {
                        window_id: ctx.window_id,
                    },
                )]);
                return true;
            }

            let EngineEvent::Ui(ui_event) = event else {
                continue;
            };
            if ui_event.document_id != control_doc_id
                || ui_event.kind != vulfram_core::core::ui::events::UiEventKind::Click
            {
                continue;
            }

            let mut cmds: Vec<EngineCmd> = Vec::new();
            match ui_event.node_id {
                id if id == btn_measure => {
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_windowed => {
                    cmds.push(window_state_cmd(ctx.window_id, EngineWindowState::Windowed));
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_maximized => {
                    cmds.push(window_state_cmd(
                        ctx.window_id,
                        EngineWindowState::Maximized,
                    ));
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_minimized => {
                    cmds.push(window_state_cmd(
                        ctx.window_id,
                        EngineWindowState::Minimized,
                    ));
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_fullscreen => {
                    cmds.push(window_state_cmd(
                        ctx.window_id,
                        EngineWindowState::Fullscreen,
                    ));
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_borderless => {
                    cmds.push(window_state_cmd(
                        ctx.window_id,
                        EngineWindowState::WindowedFullscreen,
                    ));
                    cmds.push(window_measurement_cmd(ctx.window_id));
                }
                id if id == btn_cursor_default => {
                    cmds.push(window_cursor_cmd(ctx.window_id, CursorIcon::Default));
                }
                id if id == btn_cursor_crosshair => {
                    cmds.push(window_cursor_cmd(ctx.window_id, CursorIcon::Crosshair));
                }
                _ => {}
            }
            if !cmds.is_empty() {
                let _ = send_commands(cmds);
            }
        }

        if let Some(remaining) = target_frame_time.checked_sub(now.elapsed()) {
            std::thread::sleep(remaining);
        }
    }
}
