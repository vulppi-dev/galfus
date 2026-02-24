use glam::{Mat4, Vec3};

use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdModelUpdateArgs, EnvironmentConfig, MsaaConfig, PostProcessConfig,
    SkyboxConfig, SkyboxMode,
};
use crate::core::ui::cmd::CmdUiDocumentSetRectArgs;
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use crate::demo::demo_007::maps::ENV_PROFILE_BIND_B_MSAA4;
use crate::demo::demo_007::setup::{Demo007RealmIds, Demo007Setup};
use crate::demo::send_commands;
use crate::demo::{DemoContext, run_loop_with_events};

pub fn run(ctx: DemoContext, setup: &Demo007Setup, realms: &Demo007RealmIds) -> bool {
    let window_id = ctx.window_id;
    let realm_id = realms._realm_3d;
    let ids = setup.ids;
    let base_positions = [
        Vec3::new(-4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.2, -4.0),
        Vec3::new(4.0, 0.1, 2.5),
    ];
    let mut last_msaa_slot: usize = usize::MAX;

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
                        realm_id,
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

            let msaa_slot = ((total_ms / 2000) % 3) as usize;
            if msaa_slot != last_msaa_slot {
                last_msaa_slot = msaa_slot;
                let sample_count = match msaa_slot {
                    0 => 1,
                    1 => 4,
                    _ => 8,
                };
                cmds.push(EngineCmd::CmdEnvironmentUpsert(
                    crate::core::cmd::CmdEnvironmentUpsertArgs::Update(CmdEnvironmentUpdateArgs {
                        environment_id: ENV_PROFILE_BIND_B_MSAA4,
                        config: bind_b_environment(sample_count),
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

fn bind_b_environment(sample_count: u32) -> EnvironmentConfig {
    EnvironmentConfig {
        msaa: MsaaConfig {
            enabled: sample_count > 1,
            sample_count,
        },
        skybox: SkyboxConfig {
            mode: SkyboxMode::None,
            intensity: 1.0,
            rotation: 0.0,
            ground_color: Vec3::new(0.02, 0.03, 0.04),
            horizon_color: Vec3::new(0.12, 0.16, 0.22),
            sky_color: Vec3::new(0.2, 0.35, 0.6),
            cubemap_texture_id: None,
        },
        clear_color: Vec3::new(0.0, 0.0, 0.0),
        post: PostProcessConfig {
            filter_enabled: false,
            filter_exposure: 1.0,
            filter_gamma: 2.2,
            filter_saturation: 1.0,
            filter_contrast: 1.0,
            filter_vignette: 0.15,
            filter_grain: 0.0,
            filter_chromatic_aberration: 0.0,
            filter_blur: 0.0,
            filter_sharpen: 0.2,
            filter_tonemap_mode: 1,
            outline_enabled: true,
            outline_strength: 0.0,
            outline_threshold: 0.2,
            outline_width: 1.0,
            outline_quality: 1.0,
            filter_posterize_steps: 0.0,
            cell_shading: false,
            ssao_enabled: false,
            ssao_strength: 0.0,
            ssao_radius: 1.0,
            ssao_bias: 0.02,
            ssao_power: 1.0,
            ssao_blur_radius: 0.0,
            ssao_blur_depth_threshold: 0.02,
            bloom_enabled: false,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_intensity: 0.0,
            bloom_scatter: 1.0,
        },
    }
}
