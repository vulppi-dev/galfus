use std::cell::RefCell;
use std::rc::Rc;

use glam::{Mat4, Vec3};

use crate::core::audio::{AudioPlayModeDto, CmdAudioSourcePlayArgs};
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent, PointerEvent};
use crate::core::resources::{CmdCameraUpdateArgs, CmdModelUpdateArgs};
use crate::core::system::events::SystemEvent;
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::demo_005::setup::{Demo005RealmIds, Demo005Setup};
use crate::demo::send_commands;
use crate::demo::{DemoContext, run_loop_with_events};

pub fn run(ctx: DemoContext, setup: &Demo005Setup, realms: &Demo005RealmIds) -> bool {
    let window_id = ctx.window_id;
    let ids = setup.ids;

    let state = Rc::new(RefCell::new(Demo005RuntimeState::default()));
    let state_frame = Rc::clone(&state);
    let state_event = Rc::clone(&state);

    println!(
        "Demo 005 targets: window_main={} window_aux={} window_layer_main={} window_layer_aux={} realm_plane_layer={} texture_shared={}",
        realms.target_window_main,
        realms.target_window_aux,
        realms.target_window_layer_main,
        realms.target_window_layer_aux,
        realms.target_realm_plane_layer,
        realms.target_texture_shared
    );

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let time_f = total_ms as f32 / 1000.0;
            let mut cmds = Vec::new();

            let camera_radius = 8.5;
            let camera_height = 3.0 + (time_f * 0.6).sin() * 0.6;
            let camera_angle = time_f * 0.35;
            let camera_pos = Vec3::new(
                camera_radius * camera_angle.cos(),
                camera_height,
                camera_radius * camera_angle.sin(),
            );
            let camera_transform = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y).inverse();

            cmds.push(EngineCmd::CmdCameraUpsert(
                crate::core::cmd::CmdCameraUpsertArgs::Update(CmdCameraUpdateArgs {
                    camera_id: ids.camera_id,
                    label: None,
                    transform: Some(camera_transform),
                    kind: None,
                    flags: None,
                    near_far: None,
                    layer_mask: None,
                    order: None,
                    view_position: None,
                    ortho_scale: None,
                }),
            ));

            cmds.push(EngineCmd::CmdModelUpsert(
                crate::core::cmd::CmdModelUpsertArgs::Update(CmdModelUpdateArgs {
                    window_id,
                    model_id: ids.listener_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(camera_transform * Mat4::from_scale(Vec3::splat(0.4))),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
            ));

            let wobble = time_f * 0.8;
            cmds.push(EngineCmd::CmdModelUpsert(
                crate::core::cmd::CmdModelUpsertArgs::Update(CmdModelUpdateArgs {
                    window_id,
                    model_id: 840,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(-2.0, wobble.sin() * 0.4, 0.0))
                            * Mat4::from_scale(Vec3::splat(1.0)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
            ));
            cmds.push(EngineCmd::CmdModelUpsert(
                crate::core::cmd::CmdModelUpsertArgs::Update(CmdModelUpdateArgs {
                    window_id,
                    model_id: 841,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(2.2, 0.2, -1.0))
                            * Mat4::from_euler(
                                glam::EulerRot::XYZ,
                                wobble * 0.6,
                                wobble * 0.9,
                                0.0,
                            )
                            * Mat4::from_scale(Vec3::splat(1.2)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
            ));

            {
                let mut runtime = state_frame.borrow_mut();
                if runtime.audio_ready && !runtime.audio_started {
                    runtime.audio_started = true;
                    cmds.push(EngineCmd::CmdAudioSourcePlay(CmdAudioSourcePlayArgs {
                        source_id: ids.audio_source_id,
                        resource_id: ids.audio_id,
                        timeline_id: None,
                        intensity: 1.0,
                        delay_ms: None,
                        mode: AudioPlayModeDto::Loop,
                    }));
                }
                if total_ms.saturating_sub(runtime.last_report_ms) > 1000 {
                    runtime.last_report_ms = total_ms;
                    if let Some(report) = get_profiling() {
                        if let Some(frame_report) = report.frame_report.as_ref() {
                            println!(
                                "FrameReport: order={:?} cut_edges={} blocked={} self_sampled={}",
                                frame_report.order,
                                frame_report.cut_edges.len(),
                                frame_report.blocked_connectors.len(),
                                frame_report.self_sampled_connectors.len()
                            );
                            if !frame_report.cut_edges.is_empty() {
                                println!("Cut edges: {:?}", frame_report.cut_edges);
                            }
                            println!(
                                "TargetGraph: nodes={} edges={} added={:?} removed={:?} updated={:?} binds_added={} binds_removed={} binds_updated={} plan_dirty={}",
                                frame_report.target_nodes,
                                frame_report.target_edges,
                                frame_report.target_added,
                                frame_report.target_removed,
                                frame_report.target_updated,
                                frame_report.target_layers_added.len(),
                                frame_report.target_layers_removed.len(),
                                frame_report.target_layers_updated.len(),
                                frame_report.target_plan_dirty
                            );
                            if frame_report.target_nodes < 6 {
                                println!(
                                    "Warning: expected at least 6 targets, got {}",
                                    frame_report.target_nodes
                                );
                            }
                        }
                    }
                }
            }

            cmds
        },
        move |event| {
            match event {
                EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id })
                    if id == window_id =>
                {
                    let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                        window_id: realms.window_aux,
                    })]);
                    return true;
                }
                EngineEvent::System(SystemEvent::AudioReady {
                    resource_id,
                    success,
                    message,
                }) if resource_id == ids.audio_id => {
                    let mut runtime = state_event.borrow_mut();
                    runtime.audio_ready = success;
                    println!("AudioReady: success={} message={}", success, message);
                }
                EngineEvent::Pointer(pointer_event) => {
                    log_pointer_trace(&pointer_event);
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if id == window_id => {
                    if key_code == 36 {
                        println!(
                            "KeyR pressed: host_main={} host_aux={} window_layer_main={} ui={} texture_main={} texture_aux={} conflict={}",
                            realms.host_realm_main,
                            realms.host_realm_aux,
                            realms.realm_3d_layer_main,
                            realms.realm_ui,
                            realms.realm_texture_main,
                            realms.realm_texture_aux,
                            realms.realm_conflict
                        );
                    }
                    if key_code == 106 || key_code == 94 {
                        let _ =
                            send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                                window_id: realms.window_aux,
                            })]);
                        return true;
                    }
                }
                _ => {}
            }
            false
        },
    )
}

#[derive(Debug, Default)]
struct Demo005RuntimeState {
    audio_ready: bool,
    audio_started: bool,
    last_report_ms: u64,
}

fn log_pointer_trace(event: &PointerEvent) {
    let trace = match event {
        PointerEvent::OnMove { trace, .. }
        | PointerEvent::OnEnter { trace, .. }
        | PointerEvent::OnLeave { trace, .. }
        | PointerEvent::OnButton { trace, .. }
        | PointerEvent::OnScroll { trace, .. }
        | PointerEvent::OnTouch { trace, .. }
        | PointerEvent::OnPinchGesture { trace, .. }
        | PointerEvent::OnPanGesture { trace, .. }
        | PointerEvent::OnRotationGesture { trace, .. }
        | PointerEvent::OnDoubleTapGesture { trace, .. } => trace.as_ref(),
    };

    if let Some(trace) = trace {
        println!(
            "PointerTrace: window={} realm={} target={:?} connector={:?} source_realm={:?} uv={:?}",
            trace.window_id,
            trace.realm_id,
            trace.target_id,
            trace.connector_id,
            trace.source_realm_id,
            trace.uv
        );
    }
}

fn get_profiling() -> Option<crate::core::profiling::cmd::ProfilingData> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = crate::core::vulfram_get_profiling(&mut ptr, &mut len);

    if result != crate::core::VulframResult::Success || len == 0 {
        return None;
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    Some(rmp_serde::from_slice(&bytes).expect("failed to deserialize profiling"))
}
