use glam::{Mat4, Vec3};

use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::resources::CmdModelUpdateArgs;
use crate::core::ui::cmd::CmdUiDocumentSetRectArgs;
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::demo_007::setup::Demo007Setup;
use crate::demo::send_commands;
use crate::demo::{DemoContext, run_loop_with_events};

pub fn run(ctx: DemoContext, setup: &Demo007Setup) -> bool {
    let window_id = ctx.window_id;
    let ids = setup.ids;
    let base_positions = [
        Vec3::new(-4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.2, -4.0),
        Vec3::new(4.0, 0.1, 2.5),
    ];

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let t = total_ms as f32 / 1000.0;
            let mut cmds = Vec::with_capacity(ids.model_ids.len());
            for (index, model_id) in ids.model_ids.iter().enumerate() {
                let phase = index as f32 * 0.5;
                let pos = base_positions[index];
                let transform = Mat4::from_translation(pos)
                    * Mat4::from_rotation_y(t * (0.7 + index as f32 * 0.18) + phase)
                    * Mat4::from_rotation_x(t * (0.2 + index as f32 * 0.07));

                cmds.push(EngineCmd::CmdModelUpsert(
                    crate::core::cmd::CmdModelUpsertArgs::Update(CmdModelUpdateArgs {
                        window_id,
                        model_id: *model_id,
                        label: None,
                        geometry_id: None,
                        material_id: None,
                        transform: Some(transform),
                        layer_mask: None,
                        cast_shadow: None,
                        receive_shadow: None,
                        cast_outline: None,
                        outline_color: None,
                    }),
                ));
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
                EngineEvent::Window(WindowEvent::OnResize {
                    window_id: id,
                    width,
                    height,
                }) if id == window_id => {
                    let _ = send_commands(vec![EngineCmd::CmdUiDocumentSetRect(
                        CmdUiDocumentSetRectArgs {
                            document_id: ids.ui_document_id,
                            rect: glam::Vec4::new(0.0, 0.0, width as f32, height as f32),
                        },
                    )]);
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if id == window_id => {
                    if key_code == 106 || key_code == 94 {
                        let _ =
                            send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                                window_id,
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
