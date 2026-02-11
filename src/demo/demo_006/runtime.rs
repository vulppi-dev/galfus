use std::cell::RefCell;
use std::rc::Rc;

use glam::{Mat4, Vec3};

use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::resources::CmdCameraUpdateArgs;
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::demo_006::setup::{Demo006RealmIds, Demo006Setup};
use crate::demo::send_commands;
use crate::demo::{DemoContext, run_loop_with_events};

pub fn run(ctx: DemoContext, setup: &Demo006Setup, realms: &Demo006RealmIds) -> bool {
    let window_id = ctx.window_id;
    let ids = setup.ids;

    let state = Rc::new(RefCell::new(Demo006RuntimeState::default()));
    let state_event = Rc::clone(&state);

    println!(
        "Demo 006 targets: texture_ui={} texture_view={}",
        realms.target_texture_ui, realms.target_texture_view
    );

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let time_f = total_ms as f32 / 1000.0;
            let mut cmds = Vec::new();

            let camera_radius = 8.5;
            let camera_height = 3.0 + (time_f * 0.5).sin() * 0.6;
            let camera_angle = time_f * 0.35;
            let camera_pos = Vec3::new(
                camera_radius * camera_angle.cos(),
                camera_height,
                camera_radius * camera_angle.sin(),
            );
            let camera_transform = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y).inverse();
            cmds.push(EngineCmd::CmdCameraUpdate(CmdCameraUpdateArgs {
                camera_id: ids.camera_main_id,
                label: None,
                transform: Some(camera_transform),
                kind: None,
                flags: None,
                near_far: None,
                layer_mask: None,
                order: None,
                view_position: None,
                ortho_scale: None,
            }));

            let view_angle = time_f * 0.7;
            let view_pos = Vec3::new(2.5 * view_angle.cos(), 1.8, 2.5 * view_angle.sin());
            let view_transform = Mat4::look_at_rh(view_pos, Vec3::ZERO, Vec3::Y).inverse();
            cmds.push(EngineCmd::CmdCameraUpdate(CmdCameraUpdateArgs {
                camera_id: ids.camera_view_id,
                label: None,
                transform: Some(view_transform),
                kind: None,
                flags: None,
                near_far: None,
                layer_mask: None,
                order: Some(1),
                view_position: None,
                ortho_scale: None,
            }));

            {
                let mut runtime = state_event.borrow_mut();
                if total_ms.saturating_sub(runtime.last_report_ms) > 1500 {
                    runtime.last_report_ms = total_ms;
                    if let Some(report) = get_profiling() {
                        println!(
                            "FrameReport: order={:?} cut_edges={} blocked={} self_sampled={}",
                            report.frame_report.order,
                            report.frame_report.cut_edges.len(),
                            report.frame_report.blocked_connectors.len(),
                            report.frame_report.self_sampled_connectors.len()
                        );
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
                    return true;
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if id == window_id => {
                    if key_code == 106 || key_code == 94 {
                        let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                            window_id,
                        })]);
                        return true;
                    }
                }
                EngineEvent::Ui(event) => {
                    println!(
                        "UiEvent: realm={} doc={} node={} kind={:?} label={:?}",
                        event.realm_id, event.document_id, event.node_id, event.kind, event.label
                    );
                }
                _ => {}
            }
            false
        },
    )
}

#[derive(Debug, Default)]
struct Demo006RuntimeState {
    last_report_ms: u64,
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
