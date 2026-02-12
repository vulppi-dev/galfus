use glam::{Mat4, Vec3};

use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::resources::CmdModelUpdateArgs;
use crate::core::ui::cmd::CmdUiApplyOpsArgs;
use crate::core::ui::events::UiEventKind;
use crate::core::ui::types::{UiNodeProps, UiOp};
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::demo_006::setup::{Demo006RealmIds, Demo006Setup};
use crate::demo::send_commands;
use crate::demo::{DemoContext, draw_axes_gizmos, run_loop_with_events};

pub fn run(ctx: DemoContext, setup: &Demo006Setup, _realms: &Demo006RealmIds) -> bool {
    let window_id = ctx.window_id;
    let ids = setup.ids;
    let mut ui_version = 1u64;
    let mut ui_panel_version = 1u64;
    let mut toggle_enabled = false;
    let mut counter = 0i32;
    let mut panel_counter = 0i32;

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let t = total_ms as f32 / 1000.0;
            let mut cmds = vec![EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: ids.model_cube_id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_translation(Vec3::new(0.0, 0.8, 0.0))
                        * Mat4::from_rotation_y(t * 1.6)
                        * Mat4::from_rotation_x(t * 0.7)
                        * Mat4::from_scale(Vec3::splat(1.2)),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            })];
            cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: ids.model_ui_plane_id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_translation(Vec3::new(1.0, 2.0, 3.8))
                        * Mat4::from_rotation_y(
                            std::f32::consts::PI - 0.35 + (t * 0.45).sin() * 0.08,
                        )
                        * Mat4::from_rotation_x(-0.08)
                        * Mat4::from_scale(Vec3::new(2.4, 1.0, 1.0)),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            }));
            cmds.extend(draw_axes_gizmos());
            cmds
        },
        move |event| {
            match event {
                EngineEvent::Ui(ui_event) if ui_event.document_id == ids.ui_panel_document_id => {
                    match (ui_event.node_id, ui_event.kind) {
                        (node_id, UiEventKind::ChangeCommit) if node_id == ids.ui_panel_input_id => {
                            if let Some(text) = ui_event.label {
                                ui_panel_version += 1;
                                let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                    CmdUiApplyOpsArgs {
                                        document_id: ids.ui_panel_document_id,
                                        version: ui_panel_version,
                                        ops: vec![UiOp::Set {
                                            node_id: ids.ui_panel_input_id,
                                            props: UiNodeProps::Input {
                                                value: text,
                                                placeholder: Some("Texto no UIPanel".into()),
                                                enabled: Some(true),
                                            },
                                        }],
                                    },
                                )]);
                            }
                        }
                        (node_id, UiEventKind::Click) if node_id == ids.ui_panel_button_add_id => {
                            panel_counter += 1;
                            ui_panel_version += 1;
                            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                CmdUiApplyOpsArgs {
                                    document_id: ids.ui_panel_document_id,
                                    version: ui_panel_version,
                                    ops: vec![UiOp::Set {
                                        node_id: ids.ui_panel_body_id,
                                        props: UiNodeProps::Text {
                                            text: format!("Contador painel: {panel_counter}"),
                                            size: Some(14.0),
                                            color: None,
                                        },
                                    }],
                                },
                            )]);
                        }
                        (node_id, UiEventKind::Click)
                            if node_id == ids.ui_panel_button_remove_id =>
                        {
                            panel_counter -= 1;
                            ui_panel_version += 1;
                            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                CmdUiApplyOpsArgs {
                                    document_id: ids.ui_panel_document_id,
                                    version: ui_panel_version,
                                    ops: vec![UiOp::Set {
                                        node_id: ids.ui_panel_body_id,
                                        props: UiNodeProps::Text {
                                            text: format!("Contador painel: {panel_counter}"),
                                            size: Some(14.0),
                                            color: None,
                                        },
                                    }],
                                },
                            )]);
                        }
                        _ => {}
                    }
                }
                EngineEvent::Ui(ui_event) if ui_event.document_id == ids.ui_document_id => {
                    match (ui_event.node_id, ui_event.kind) {
                        (node_id, UiEventKind::ChangeCommit) if node_id == ids.ui_input_id => {
                            if let Some(text) = ui_event.label {
                                ui_version += 1;
                                let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                    CmdUiApplyOpsArgs {
                                        document_id: ids.ui_document_id,
                                        version: ui_version,
                                        ops: vec![UiOp::Set {
                                            node_id: ids.ui_input_id,
                                            props: UiNodeProps::Input {
                                                value: text,
                                                placeholder: Some("Digite algo...".into()),
                                                enabled: Some(true),
                                            },
                                        }],
                                    },
                                )]);
                            }
                        }
                        (node_id, UiEventKind::Click) if node_id == ids.ui_toggle_checkbox_id => {
                            toggle_enabled = !toggle_enabled;
                            ui_version += 1;
                            let toggle_label = if toggle_enabled {
                                "[x] Habilitar efeito"
                            } else {
                                "[ ] Habilitar efeito"
                            };
                            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                CmdUiApplyOpsArgs {
                                    document_id: ids.ui_document_id,
                                    version: ui_version,
                                    ops: vec![UiOp::Set {
                                        node_id: ids.ui_toggle_checkbox_id,
                                        props: UiNodeProps::Button {
                                            label: toggle_label.into(),
                                            enabled: Some(true),
                                        },
                                    }],
                                },
                            )]);
                        }
                        (node_id, UiEventKind::Click) if node_id == ids.ui_button_add_id => {
                            counter += 1;
                            ui_version += 1;
                            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                CmdUiApplyOpsArgs {
                                    document_id: ids.ui_document_id,
                                    version: ui_version,
                                    ops: vec![UiOp::Set {
                                        node_id: ids.ui_body_id,
                                        props: UiNodeProps::Text {
                                            text: format!("Contador: {counter}"),
                                            size: Some(15.0),
                                            color: None,
                                        },
                                    }],
                                },
                            )]);
                        }
                        (node_id, UiEventKind::Click) if node_id == ids.ui_button_remove_id => {
                            counter -= 1;
                            ui_version += 1;
                            let _ = send_commands(vec![EngineCmd::CmdUiApplyOps(
                                CmdUiApplyOpsArgs {
                                    document_id: ids.ui_document_id,
                                    version: ui_version,
                                    ops: vec![UiOp::Set {
                                        node_id: ids.ui_body_id,
                                        props: UiNodeProps::Text {
                                            text: format!("Contador: {counter}"),
                                            size: Some(15.0),
                                            color: None,
                                        },
                                    }],
                                },
                            )]);
                        }
                        _ => {}
                    }
                }
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
                _ => {}
            }
            false
        },
    )
}
