use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use glam::{Mat4, Vec3, Vec4};

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs,
    EnvironmentConfig, MsaaConfig, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode,
};
use crate::core::system::events::SystemEvent;
use crate::core::target::cmd::{CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs};
use crate::core::target::{DimensionValue, TargetKind, TargetLayerLayout};
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::{
    DemoContext, create_ambient_light_cmd, create_camera_cmd, create_floor_cmd,
    create_point_light_cmd, create_shadow_config_cmd, create_standard_material_cmd, create_window,
    run_loop_with_events,
};

/// Demo 013 is dedicated to stress topology validation:
/// - 2 windows / 2 host realms
/// - extra realms/targets/layers
/// - intentional conflicting layer routes
/// This demo is not a performance baseline.
pub fn run(ctx: DemoContext) -> bool {
    let setup = setup_stress(ctx);
    run_stress_loop(ctx, setup)
}

#[derive(Debug, Clone, Copy)]
struct Demo013Setup {
    window_aux: u32,
    host_realm_main: u32,
    host_realm_aux: u32,
    camera_main_id: u32,
    camera_aux_id: u32,
}

#[derive(Debug, Clone, Copy)]
struct Demo013TargetIds {
    window_main: u64,
    window_aux: u64,
    window_layer_main: u64,
    window_layer_aux: u64,
    realm_plane_layer: u64,
    texture_shared: u64,
}

#[derive(Debug, Clone, Copy)]
struct Demo013LayerRealms {
    host_main: u32,
    host_aux: u32,
    window_layer_main: u32,
    ui: u32,
    texture_main: u32,
    texture_aux: u32,
    conflict: u32,
}

fn setup_stress(ctx: DemoContext) -> Demo013Setup {
    drain_responses();

    let window_main = ctx.window_id;
    let host_realm_main = ctx.realm_id;
    let window_aux = 2;
    let host_realm_aux = create_window(window_aux, "Vulfram Demo 013 Aux").realm_id;

    let create_realm_cmds = vec![
        EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(window_main),
            importance: None,
            cache_policy: None,
            flags: None,
        }),
        EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(window_main),
            importance: None,
            cache_policy: None,
            flags: None,
        }),
        EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(window_main),
            importance: None,
            cache_policy: None,
            flags: None,
        }),
        EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(window_aux),
            importance: None,
            cache_policy: None,
            flags: None,
        }),
        EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
            kind: RealmKindDto::TwoD,
            output_surface_id: None,
            host_window_id: Some(window_main),
            importance: None,
            cache_policy: None,
            flags: None,
        }),
    ];
    assert_eq!(send_commands(create_realm_cmds), VulframResult::Success);
    let created = wait_for_realm_creates(5);

    let layer_realms = Demo013LayerRealms {
        host_main: host_realm_main,
        host_aux: host_realm_aux,
        window_layer_main: created[0],
        ui: created[1],
        texture_main: created[2],
        texture_aux: created[3],
        conflict: created[4],
    };

    let camera_main_id = 910;
    let camera_aux_id = 911;
    let scene_cmds = build_scene_cmds(window_main, window_aux, camera_main_id, camera_aux_id);
    assert_eq!(send_commands(scene_cmds), VulframResult::Success);

    let (targets, target_cmds) = build_target_cmds(window_main, window_aux);
    assert_eq!(send_commands(target_cmds), VulframResult::Success);
    wait_for_target_upserts(6);

    let layer_cmds = build_layer_cmds(targets, layer_realms, camera_main_id, camera_aux_id);
    assert_eq!(send_commands(layer_cmds), VulframResult::Success);
    wait_for_target_layer_upserts(11);

    println!(
        "Demo 013 stress realms: host_main={} host_aux={} window_main={} window_aux={} conflict={}",
        host_realm_main, host_realm_aux, window_main, window_aux, layer_realms.conflict
    );

    Demo013Setup {
        window_aux,
        host_realm_main,
        host_realm_aux,
        camera_main_id,
        camera_aux_id,
    }
}

fn run_stress_loop(ctx: DemoContext, setup: Demo013Setup) -> bool {
    let window_id = ctx.window_id;
    let state = Rc::new(RefCell::new(0_u64));
    let state_frame = Rc::clone(&state);

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let mut last_report_ms = state_frame.borrow_mut();
            if total_ms.saturating_sub(*last_report_ms) > 1000 {
                *last_report_ms = total_ms;
                if let Some(report) = get_profiling() {
                    if let Some(frame_report) = report.frame_report.as_ref() {
                        println!(
                            "Demo013 FrameReport: order={:?} cut_edges={} blocked={} self_sampled={}",
                            frame_report.order,
                            frame_report.cut_edges.len(),
                            frame_report.blocked_connectors.len(),
                            frame_report.self_sampled_connectors.len()
                        );
                    }
                }
            }
            Vec::new()
        },
        move |event| {
            match event {
                EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id })
                    if id == window_id =>
                {
                    let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                        window_id: setup.window_aux,
                    })]);
                    return true;
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if id == window_id => {
                    if key_code == 36 {
                        println!(
                            "Demo013 KeyR: host_main={} host_aux={} cam_main={} cam_aux={}",
                            setup.host_realm_main,
                            setup.host_realm_aux,
                            setup.camera_main_id,
                            setup.camera_aux_id
                        );
                    }
                    if key_code == 106 || key_code == 94 {
                        let _ =
                            send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                                window_id: setup.window_aux,
                            })]);
                        return true;
                    }
                }
                EngineEvent::System(SystemEvent::Error { scope, message, .. }) => {
                    println!("Demo013 SystemError: scope={} message={}", scope, message);
                }
                _ => {}
            }
            false
        },
    )
}

fn build_target_cmds(window_main: u32, window_aux: u32) -> (Demo013TargetIds, Vec<EngineCmd>) {
    let target_ids = Demo013TargetIds {
        window_main: 9100,
        window_aux: 9101,
        window_layer_main: 9102,
        realm_plane_layer: 9103,
        window_layer_aux: 9104,
        texture_shared: 9105,
    };

    let targets = vec![
        CmdTargetUpsertArgs {
            target_id: target_ids.window_main,
            kind: TargetKind::Window,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.window_aux,
            kind: TargetKind::Window,
            window_id: Some(window_aux),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.window_layer_main,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: Some(4),
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.realm_plane_layer,
            kind: TargetKind::RealmPlane,
            window_id: Some(window_main),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.window_layer_aux,
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(window_aux),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
        CmdTargetUpsertArgs {
            target_id: target_ids.texture_shared,
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(glam::UVec2::new(256, 256)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    ];

    (
        target_ids,
        targets
            .into_iter()
            .map(EngineCmd::CmdTargetUpsert)
            .collect(),
    )
}

fn build_scene_cmds(
    window_main: u32,
    window_aux: u32,
    camera_main_id: u32,
    camera_aux_id: u32,
) -> Vec<EngineCmd> {
    let post = PostProcessConfig {
        filter_enabled: false,
        filter_exposure: 1.0,
        filter_gamma: 2.2,
        filter_saturation: 1.0,
        filter_contrast: 1.0,
        filter_vignette: 0.0,
        filter_grain: 0.0,
        filter_chromatic_aberration: 0.0,
        filter_blur: 0.0,
        filter_sharpen: 0.1,
        filter_tonemap_mode: 1,
        filter_posterize_steps: 0.0,
        cell_shading: false,
        outline_enabled: true,
        outline_strength: 0.4,
        outline_threshold: 0.2,
        outline_width: 1.0,
        outline_quality: 1.0,
        ssao_enabled: true,
        ssao_strength: 0.5,
        ssao_radius: 0.8,
        ssao_bias: 0.02,
        ssao_power: 1.0,
        ssao_blur_radius: 2.0,
        ssao_blur_depth_threshold: 0.02,
        bloom_enabled: false,
        bloom_threshold: 1.0,
        bloom_knee: 0.8,
        bloom_intensity: 0.0,
        bloom_scatter: 1.0,
    };

    vec![
        EngineCmd::CmdEnvironmentUpsert(crate::core::cmd::CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: window_main,
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
                    clear_color: Vec3::new(0.0, 0.0, 0.0),
                    post: post.clone(),
                },
            },
        )),
        EngineCmd::CmdEnvironmentUpsert(crate::core::cmd::CmdEnvironmentUpsertArgs::Update(
            CmdEnvironmentUpdateArgs {
                environment_id: window_aux,
                config: EnvironmentConfig {
                    msaa: MsaaConfig {
                        enabled: true,
                        sample_count: 4,
                    },
                    skybox: SkyboxConfig {
                        mode: SkyboxMode::Procedural,
                        intensity: 0.8,
                        rotation: 0.0,
                        ground_color: Vec3::new(0.04, 0.04, 0.05),
                        horizon_color: Vec3::new(0.18, 0.2, 0.3),
                        sky_color: Vec3::new(0.14, 0.24, 0.4),
                        cubemap_texture_id: None,
                    },
                    clear_color: Vec3::new(0.0, 0.0, 0.0),
                    post,
                },
            },
        )),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id: window_main,
            geometry_id: 9200,
            label: Some("Demo 013 Main Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id: window_main,
            geometry_id: 9201,
            label: Some("Demo 013 Main Floor".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id: window_aux,
            geometry_id: 9210,
            label: Some("Demo 013 Aux Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id: window_aux,
            geometry_id: 9211,
            label: Some("Demo 013 Aux Floor".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        create_camera_cmd(
            camera_main_id,
            "Demo 013 Main Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 3.0, 8.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_camera_cmd(
            camera_aux_id,
            "Demo 013 Aux Camera",
            Mat4::look_at_rh(Vec3::new(1.5, 2.6, 6.2), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_main, 9220, Vec4::new(3.0, 5.0, 2.0, 1.0)),
        create_ambient_light_cmd(window_main, 9221, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.7),
        create_point_light_cmd(window_aux, 9230, Vec4::new(2.0, 4.0, 2.0, 1.0)),
        create_ambient_light_cmd(window_aux, 9231, Vec4::new(0.35, 0.35, 0.35, 1.0), 0.7),
        create_standard_material_cmd(
            window_main,
            9240,
            "Demo 013 Main Mat",
            Vec4::new(0.2, 0.6, 0.9, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            window_main,
            9241,
            "Demo 013 Main Floor Mat",
            Vec4::new(0.4, 0.4, 0.45, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            window_aux,
            9250,
            "Demo 013 Aux Mat",
            Vec4::new(0.15, 0.5, 0.25, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            window_aux,
            9251,
            "Demo 013 Aux Floor Mat",
            Vec4::new(0.35, 0.35, 0.4, 1.0),
            None,
            None,
        ),
        EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
            CmdModelCreateArgs {
                window_id: window_main,
                model_id: 9260,
                label: Some("Demo 013 Main Model".into()),
                geometry_id: 9200,
                material_id: Some(9240),
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
                    * Mat4::from_scale(Vec3::splat(1.2)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.9, 0.3, 0.2, 1.0),
            },
        )),
        create_floor_cmd(window_main, 9201, 9241),
        create_shadow_config_cmd(window_main),
        EngineCmd::CmdModelUpsert(crate::core::cmd::CmdModelUpsertArgs::Create(
            CmdModelCreateArgs {
                window_id: window_aux,
                model_id: 9270,
                label: Some("Demo 013 Aux Model".into()),
                geometry_id: 9210,
                material_id: Some(9250),
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
                    * Mat4::from_scale(Vec3::splat(1.4)),
                layer_mask: 0xFFFFFFFF,
                cast_shadow: true,
                receive_shadow: true,
                cast_outline: true,
                outline_color: Vec4::new(0.1, 0.9, 0.4, 1.0),
            },
        )),
        create_floor_cmd(window_aux, 9211, 9251),
        create_shadow_config_cmd(window_aux),
    ]
}

fn build_layer_cmds(
    targets: Demo013TargetIds,
    realms: Demo013LayerRealms,
    camera_main_id: u32,
    camera_aux_id: u32,
) -> Vec<EngineCmd> {
    let layers = vec![
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_main,
            layout: TargetLayerLayout::default(),
            camera_id: Some(camera_main_id),
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.window_aux,
            layout: TargetLayerLayout::default(),
            camera_id: Some(camera_aux_id),
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.window_layer_main,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(40.0, 40.0, 320.0, 220.0), 2, 0, None),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.ui,
            target_id: targets.realm_plane_layer,
            layout: layer_layout(
                Vec4::new(720.0, 120.0, 220.0, 180.0),
                3,
                1,
                Some(Vec4::new(720.0, 120.0, 160.0, 140.0)),
            ),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_main,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.texture_aux,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.window_layer_main,
            layout: TargetLayerLayout::default(),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.conflict,
            target_id: targets.texture_shared,
            layout: TargetLayerLayout::default(),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(60.0, 360.0, 220.0, 160.0), 1, 0, None),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_main,
            target_id: targets.window_layer_aux,
            layout: layer_layout(Vec4::new(1020.0, 40.0, 180.0, 120.0), 0, 0, None),
            camera_id: None,
            environment_id: None,
        },
        CmdTargetLayerUpsertArgs {
            realm_id: realms.host_aux,
            target_id: targets.window_layer_main,
            layout: layer_layout(Vec4::new(20.0, 40.0, 200.0, 140.0), 0, 0, None),
            camera_id: None,
            environment_id: None,
        },
    ];

    layers
        .into_iter()
        .map(EngineCmd::CmdTargetLayerUpsert)
        .collect()
}

fn layer_layout(
    rect: Vec4,
    z_index: i32,
    blend_mode: u32,
    clip: Option<Vec4>,
) -> TargetLayerLayout {
    TargetLayerLayout {
        left: DimensionValue::Px(rect.x),
        top: DimensionValue::Px(rect.y),
        width: DimensionValue::Px(rect.z),
        height: DimensionValue::Px(rect.w),
        z_index,
        blend_mode,
        clip,
    }
}

fn drain_responses() {
    for _ in 0..16 {
        if receive_responses().is_empty() {
            break;
        }
    }
}

fn wait_for_realm_creates(expected: usize) -> Vec<u32> {
    let mut realms = Vec::new();
    for _ in 0..240 {
        for response in receive_responses() {
            if let CommandResponse::RealmCreate(result) = response.response {
                assert!(result.success, "[demo013:realm-create] {}", result.message);
                if let Some(realm_id) = result.realm_id {
                    realms.push(realm_id);
                }
            }
        }
        if realms.len() >= expected {
            realms.truncate(expected);
            return realms;
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("[demo013:realm-create] missing responses");
}

fn wait_for_target_upserts(expected: usize) {
    wait_for_responses(expected, "[demo013:target-upsert]", |response| {
        if let CommandResponse::TargetUpsert(result) = response {
            assert!(result.success, "[demo013:target-upsert] {}", result.message);
            true
        } else {
            false
        }
    });
}

fn wait_for_target_layer_upserts(expected: usize) {
    wait_for_responses(expected, "[demo013:target-layer-upsert]", |response| {
        if let CommandResponse::TargetLayerUpsert(result) = response {
            assert!(
                result.success,
                "[demo013:target-layer-upsert] {}",
                result.message
            );
            true
        } else {
            false
        }
    });
}

fn wait_for_responses<F>(expected: usize, context: &str, mut match_fn: F)
where
    F: FnMut(CommandResponse) -> bool,
{
    let mut count = 0usize;
    for _ in 0..240 {
        for response in receive_responses() {
            if match_fn(response.response) {
                count += 1;
            }
        }
        if count >= expected {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(crate::core::vulfram_tick(1, 16), VulframResult::Success);
    }
    panic!("{} missing responses", context);
}

fn get_profiling() -> Option<crate::core::profiling::cmd::ProfilingData> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = crate::core::vulfram_get_profiling(&mut ptr, &mut len);

    if result != crate::core::VulframResult::Success || len == 0 {
        return None;
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    rmp_serde::from_slice(&bytes).ok()
}
