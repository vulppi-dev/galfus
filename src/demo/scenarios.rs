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
use crate::core::input::keycodes::{KEY_ESCAPE, KEY_W};
use crate::core::profiling::state::ProfilingDetailLevel;
use crate::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use crate::core::render::gizmos::{CmdGizmoDrawAabbArgs, CmdGizmoDrawLineArgs};
use crate::core::resources::geometry::{CmdGeometryCreateArgs, GeometryPrimitiveEntry};
use crate::core::resources::shadow::CmdShadowConfigureArgs;
use crate::core::resources::shadow::ShadowConfig;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdCameraDisposeArgs, CmdCameraListArgs,
    CmdEnvironmentUpdateArgs, CmdGeometryDisposeArgs, CmdGeometryListArgs, CmdLightCreateArgs,
    CmdLightDisposeArgs, CmdLightListArgs, CmdMaterialCreateArgs, CmdMaterialDisposeArgs,
    CmdMaterialListArgs, CmdModelCreateArgs, CmdModelListArgs, CmdModelUpdateArgs,
    CmdPoseUpdateArgs, CmdPrimitiveGeometryCreateArgs, CmdTextureBindTargetArgs,
    CmdTextureCreateFromBufferArgs, CmdTextureCreateSolidColorArgs, CmdTextureDisposeArgs,
    CmdTextureListArgs, EnvironmentConfig, LightKind, MaterialKind, MaterialOptions, MsaaConfig,
    PbrOptions, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode, TextureCreateMode,
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
use crate::demo::io::{receive_events, receive_responses, send_command, send_commands};
use crate::demo::loop_utils::run_loop_with_events;
use crate::demo::session::create_window;
use crate::demo::{DemoContext, DemoKind};
use pointer_listener_lab::run_demo_7_pointer_listener_lab;
use setup::{
    aux_window_commands, base_scene_commands, extra_setup_commands, list_commands, ui_button_op,
    window_cursor_cmd, window_measurement_cmd, window_state_cmd,
};
use window_ui::run_demo_2_window_ui;

mod pointer_listener_lab;
mod pointer_listener_lab_ui;
mod setup;
mod window_ui;

#[derive(Clone, Copy)]
struct DemoIds {
    camera_id: u32,
    geometry_id: u32,
    ground_geometry_id: u32,
    material_id: u32,
    ground_material_id: u32,
    model_id: u32,
    ground_model_id: u32,
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
            ground_geometry_id: base + 12,
            material_id: base + 3,
            ground_material_id: base + 13,
            model_id: base + 4,
            ground_model_id: base + 14,
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
    match demo {
        DemoKind::Demo1 => run_demo_1(ctx),
        DemoKind::Demo2 => run_demo_2_window_ui(ctx),
        DemoKind::Demo3 => run_demo_3(ctx),
        DemoKind::Demo4 => run_demo_4(ctx),
        DemoKind::Demo5 => run_demo_5(ctx),
        DemoKind::Demo6 => run_demo_6(ctx),
        DemoKind::Demo7 => run_demo_7_pointer_listener_lab(ctx),
    }
}

#[derive(Clone, Copy)]
struct BundleOptions {
    create_temp_realm: bool,
    enable_aux_window: bool,
    draw_gizmos: bool,
    poll_lists: bool,
    log_input_wait: bool,
}

fn run_demo_1(ctx: DemoContext) -> bool {
    run_demo_bundle(
        ctx,
        1,
        &[1, 2, 3, 4, 5],
        BundleOptions {
            create_temp_realm: true,
            enable_aux_window: false,
            draw_gizmos: false,
            poll_lists: false,
            log_input_wait: false,
        },
    )
}

fn run_demo_3(ctx: DemoContext) -> bool {
    run_demo_bundle(
        ctx,
        3,
        &[10, 11, 12, 13, 14, 15, 16],
        BundleOptions {
            create_temp_realm: false,
            enable_aux_window: false,
            draw_gizmos: false,
            poll_lists: false,
            log_input_wait: false,
        },
    )
}

fn run_demo_4(ctx: DemoContext) -> bool {
    run_demo_bundle(
        ctx,
        4,
        &[17, 18, 19, 20, 21],
        BundleOptions {
            create_temp_realm: false,
            enable_aux_window: false,
            draw_gizmos: true,
            poll_lists: false,
            log_input_wait: false,
        },
    )
}

fn run_demo_5(ctx: DemoContext) -> bool {
    run_demo_bundle(
        ctx,
        5,
        &[22, 23, 24, 25],
        BundleOptions {
            create_temp_realm: false,
            enable_aux_window: true,
            draw_gizmos: false,
            poll_lists: false,
            log_input_wait: true,
        },
    )
}

fn run_demo_6(ctx: DemoContext) -> bool {
    run_demo_bundle(
        ctx,
        6,
        &[26, 27, 28],
        BundleOptions {
            create_temp_realm: false,
            enable_aux_window: true,
            draw_gizmos: false,
            poll_lists: true,
            log_input_wait: false,
        },
    )
}

fn run_demo_bundle(
    ctx: DemoContext,
    demo_number: u32,
    scenarios: &[u32],
    options: BundleOptions,
) -> bool {
    let ids = DemoIds::from_number(demo_number);
    let ui_realm_id = ctx.realm_id;
    if options.create_temp_realm {
        _ = create_and_dispose_temp_realm(ctx.window_id);
    }

    let mut hud = FpsHud::new(demo_number);
    let mut setup_cmds = base_scene_commands(ctx, ids);
    setup_cmds.extend(hud.setup_commands(ui_realm_id));

    let mut aux_windows: Vec<u32> = Vec::new();
    let mut aux_huds: Vec<(u32, FpsHud)> = Vec::new();
    if options.enable_aux_window {
        let aux_window_id = ctx.window_id + 1;
        let aux_binding =
            create_window(aux_window_id, &format!("Vulfram Demo {} Aux", demo_number));
        setup_cmds.extend(aux_window_commands(
            aux_window_id,
            aux_binding.realm_id,
            ids,
        ));
        let aux_hud = FpsHud::new(1_000 + demo_number);
        setup_cmds.extend(aux_hud.setup_commands(aux_binding.realm_id));
        aux_huds.push((aux_window_id, aux_hud));
        aux_windows.push(aux_window_id);
    }

    for scenario in scenarios {
        setup_cmds.extend(extra_setup_commands(*scenario, ctx, ui_realm_id, ids));
    }
    let _ = send_commands(setup_cmds);
    let _ = receive_responses();

    let mut last_list_ms = 0_u64;
    run_loop_with_events(
        ctx.window_id,
        None,
        move |total_ms, delta_ms| {
            let mut cmds = hud.frame_commands(total_ms, delta_ms);
            for (_, aux_hud) in &mut aux_huds {
                cmds.extend(aux_hud.frame_commands(total_ms, delta_ms));
            }
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

            if options.draw_gizmos {
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

            if options.poll_lists && total_ms.saturating_sub(last_list_ms) >= 1000 {
                last_list_ms = total_ms;
                cmds.extend(list_commands(ctx.window_id));
            }

            if options.log_input_wait && total_ms.saturating_sub(last_list_ms) >= 1500 {
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
        }) if *id == window_id => *key_code == KEY_ESCAPE || (*key_code == KEY_W && modifiers.ctrl),
        _ => false,
    }
}

fn create_ui_realm(_window_id: u32) -> Option<u32> {
    let _ = receive_responses();
    let _ = receive_events();
    let command_id = send_command(EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        importance: None,
        cache_policy: None,
        flags: None,
    }))?;

    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
            if envelope.id != command_id {
                continue;
            }
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

fn create_and_dispose_temp_realm(_window_id: u32) -> bool {
    let _ = receive_responses();
    let _ = receive_events();
    let Some(command_id) = send_command(EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        importance: None,
        cache_policy: None,
        flags: None,
    })) else {
        return false;
    };

    let mut realm_id: Option<u32> = None;
    for attempt in 0_u64..120 {
        let _ = core::vulfram_tick(attempt * 16, 16);
        for envelope in receive_responses() {
            if envelope.id != command_id {
                continue;
            }
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
