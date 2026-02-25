use crate::core::audio::{
    AudioPlayModeDto, AudioSourceTransportActionDto, CmdAudioSourceTransportArgs,
};
use crate::core::cmd::EngineCmd;
use crate::core::cmd::EngineEvent;
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::resources::{
    CmdCameraUpdateArgs, CmdModelUpdateArgs, CmdTextureCreateFromBufferArgs, TextureCreateMode,
};
use crate::core::system::events::SystemEvent;
use crate::demo::demo_004::setup::Demo004Setup;
use crate::demo::{
    DemoContext, load_texture_bytes, run_loop_with_events, send_commands, upload_texture_bytes,
};
use glam::{Mat4, Vec3};
use std::cell::RefCell;
use std::rc::Rc;

pub fn run(ctx: DemoContext, setup: &Demo004Setup) -> bool {
    let window_id = ctx.window_id;
    let realm_id = ctx.realm_id;
    let ids = setup.ids;
    let cube_models = &setup.cube_models;

    let skybox_bytes = load_texture_bytes("assets/skybox.exr");

    let audio_state = Rc::new(RefCell::new((false, false, false)));

    let audio_state_frame = Rc::clone(&audio_state);
    let audio_state_events = Rc::clone(&audio_state);

    upload_texture_bytes(&skybox_bytes, ids.skybox_buffer_id);
    let _ = send_commands(vec![EngineCmd::CmdTextureCreateFromBuffer(
        CmdTextureCreateFromBufferArgs {
            texture_id: ids.skybox_texture_id,
            label: Some("Skybox Texture".into()),
            buffer_id: ids.skybox_buffer_id,
            srgb: Some(false),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        },
    )]);
    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let time_f = total_ms as f32 / 1000.0;
            let mut cmds = Vec::new();
            let camera_radius = 8.0;
            let camera_base_height = 3.0;
            let camera_angle = time_f * 0.35;
            let camera_height = camera_base_height + (time_f * 0.7).sin() * 1.25;
            let camera_pos = Vec3::new(
                camera_radius * camera_angle.cos(),
                camera_height,
                camera_radius * camera_angle.sin(),
            );
            let camera_transform = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y).inverse();
            cmds.push(EngineCmd::CmdCameraUpsert(
                crate::core::cmd::CmdCameraUpsertArgs::Update(CmdCameraUpdateArgs {
                    realm_id,
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
                    realm_id,
                    model_id: ids.listener_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(Mat4::from_translation(camera_pos)),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
            ));
            for (index, (model_id, base_pos, _outline)) in cube_models.iter().enumerate() {
                let phase = time_f * 0.35 + index as f32 * 0.8;
                let vertical = if index >= 4 { phase.sin() * 0.08 } else { 0.0 };
                let rotation = if index % 2 == 0 {
                    phase * 0.25
                } else {
                    -phase * 0.2
                };
                let transform = Mat4::from_translation(*base_pos + Vec3::new(0.0, vertical, 0.0))
                    * Mat4::from_rotation_y(rotation)
                    * Mat4::from_scale(Vec3::splat(1.05));
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
            {
                let mut state = audio_state_frame.borrow_mut();
                if state.0 && state.1 != state.2 {
                    state.2 = state.1;
                    if state.1 {
                        cmds.push(EngineCmd::CmdAudioSourceTransport(
                            CmdAudioSourceTransportArgs {
                                source_id: ids.audio_source_id,
                                action: AudioSourceTransportActionDto::Play,
                                resource_id: Some(ids.audio_id),
                                timeline_id: None,
                                intensity: Some(1.0),
                                delay_ms: None,
                                mode: Some(AudioPlayModeDto::Loop),
                            },
                        ));
                    } else {
                        cmds.push(EngineCmd::CmdAudioSourceTransport(
                            CmdAudioSourceTransportArgs {
                                source_id: ids.audio_source_id,
                                action: AudioSourceTransportActionDto::Stop,
                                resource_id: None,
                                timeline_id: None,
                                intensity: None,
                                delay_ms: None,
                                mode: None,
                            },
                        ));
                    }
                }
            }

            cmds
        },
        move |event| {
            match &event {
                EngineEvent::System(SystemEvent::AudioReady {
                    resource_id: ready_id,
                    success,
                    message,
                }) if *ready_id == ids.audio_id => {
                    let mut state = audio_state_events.borrow_mut();
                    state.0 = *success;
                    println!("AudioReady: success={} message={}", success, message);
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if *id == window_id && *key_code == 62 => {
                    let mut state = audio_state_events.borrow_mut();
                    state.1 = !state.1;
                }
                EngineEvent::System(SystemEvent::TextureReady {
                    window_id: ready_window,
                    texture_id: ready_texture,
                    success,
                    message,
                }) if *ready_window == window_id && *ready_texture == ids.skybox_texture_id => {
                    if !success {
                        println!("Skybox texture failed: {}", message);
                    }
                }
                _ => {}
            }
            false
        },
    )
}
