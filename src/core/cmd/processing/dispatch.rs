use super::super::*;
use crate::core::platforms::PlatformProxy;
use crate::core::state::EngineState;

fn mark_windows_dirty(engine: &mut EngineState) {
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
}

pub(super) fn dispatch_command(
    engine: &mut EngineState,
    platform: &mut dyn PlatformProxy,
    pack: EngineCmdEnvelope,
) {
    match pack.cmd {
        EngineCmd::CmdNotificationSend(args) => {
            let result =
                sys::engine_cmd_notification_send(engine, platform.event_loop_proxy(), &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::NotificationSend(result),
            });
        }
        EngineCmd::CmdSystemDiagnosticsSet(args) => {
            let result = sys::engine_cmd_system_diagnostics_set(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::SystemDiagnosticsSet(result),
            });
        }
        EngineCmd::CmdWindowCreate(args) => {
            match platform.handle_window_create(engine, pack.id, &args) {
                Ok(()) => {}
                Err(result) => {
                    engine.response_queue.push(CommandResponseEnvelope {
                        id: pack.id,
                        response: CommandResponse::WindowCreate(result),
                    });
                }
            }
        }
        EngineCmd::CmdWindowClose(args) => {
            let result = win::engine_cmd_window_close(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowClose(result),
            });
        }
        EngineCmd::CmdWindowMeasurement(args) => {
            let result = win::engine_cmd_window_measurement(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowMeasurement(result),
            });
        }
        EngineCmd::CmdWindowCursor(args) => {
            let result = win::engine_cmd_window_cursor(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowCursor(result),
            });
        }
        EngineCmd::CmdWindowState(args) => {
            let result = win::engine_cmd_window_state(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowState(result),
            });
        }
        EngineCmd::CmdInputTargetListenerUpsert(args) => {
            let result = crate::core::input::listeners::engine_cmd_input_target_listener_upsert(
                engine, &args,
            );
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::InputTargetListenerUpsert(result),
            });
        }
        EngineCmd::CmdInputTargetListenerDispose(args) => {
            let result = crate::core::input::listeners::engine_cmd_input_target_listener_dispose(
                engine, &args,
            );
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::InputTargetListenerDispose(result),
            });
        }
        EngineCmd::CmdInputTargetListenerList(args) => {
            let result =
                crate::core::input::listeners::engine_cmd_input_target_listener_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::InputTargetListenerList(result),
            });
        }
        EngineCmd::CmdUploadBufferDiscardAll(args) => {
            let result = buf::engine_cmd_upload_buffer_discard_all(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::UploadBufferDiscardAll(result),
            });
        }
        EngineCmd::CmdCameraUpsert(args) => {
            let result = match args {
                CmdCameraUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_camera_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdCameraUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_camera_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::CameraUpsert(result),
            });
        }
        EngineCmd::CmdCameraDispose(args) => {
            let result = res::engine_cmd_camera_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::CameraDispose(result),
            });
        }
        EngineCmd::CmdModelUpsert(args) => {
            let result = match args {
                CmdModelUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_model_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdModelUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_model_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::ModelUpsert(result),
            });
        }
        EngineCmd::CmdPoseUpdate(args) => {
            let result = res::engine_cmd_pose_update(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::PoseUpdate(result),
            });
        }
        EngineCmd::CmdModelDispose(args) => {
            let result = res::engine_cmd_model_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::ModelDispose(result),
            });
        }
        EngineCmd::CmdLightUpsert(args) => {
            let result = match args {
                CmdLightUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_light_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdLightUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_light_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::LightUpsert(result),
            });
        }
        EngineCmd::CmdLightDispose(args) => {
            let result = res::engine_cmd_light_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::LightDispose(result),
            });
        }
        EngineCmd::CmdMaterialUpsert(args) => {
            let result = match args {
                CmdMaterialUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_material_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdMaterialUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_material_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialUpsert(result),
            });
        }
        EngineCmd::CmdMaterialDispose(args) => {
            let result = res::engine_cmd_material_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialDispose(result),
            });
        }
        EngineCmd::CmdTextureCreateFromBuffer(args) => {
            let result = res::engine_cmd_texture_create_from_buffer(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureCreateFromBuffer(result),
            });
        }
        EngineCmd::CmdTextureCreateSolidColor(args) => {
            let result = res::engine_cmd_texture_create_solid_color(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureCreateSolidColor(result),
            });
        }
        EngineCmd::CmdTextureDispose(args) => {
            let result = res::engine_cmd_texture_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureDispose(result),
            });
        }
        EngineCmd::CmdTextureBindTarget(args) => {
            let result = res::engine_cmd_texture_bind_target(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureBindTarget(result),
            });
        }
        EngineCmd::CmdAudioListenerUpsert(args) => {
            let result = match args {
                CmdAudioListenerUpsertArgs::Create(create_args) => {
                    let create_result =
                        audio::engine_cmd_audio_listener_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdAudioListenerUpsertArgs::Update(update_args) => {
                    let update_result =
                        audio::engine_cmd_audio_listener_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioListenerUpsert(result),
            });
        }
        EngineCmd::CmdAudioListenerDispose(args) => {
            let result = audio::engine_cmd_audio_listener_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioListenerDispose(result),
            });
        }
        EngineCmd::CmdAudioResourceUpsert(args) => {
            let result = audio::engine_cmd_audio_resource_upsert(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioResourceUpsert(result),
            });
        }
        EngineCmd::CmdAudioSourceUpsert(args) => {
            let result = match args {
                CmdAudioSourceUpsertArgs::Create(create_args) => {
                    let create_result = audio::engine_cmd_audio_source_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdAudioSourceUpsertArgs::Update(update_args) => {
                    let update_result = audio::engine_cmd_audio_source_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceUpsert(result),
            });
        }
        EngineCmd::CmdAudioSourceTransport(args) => {
            let result = audio::engine_cmd_audio_source_transport(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceTransport(result),
            });
        }
        EngineCmd::CmdAudioStateGet(args) => {
            let result = audio::engine_cmd_audio_state_get(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioStateGet(result),
            });
        }
        EngineCmd::CmdAudioSourceDispose(args) => {
            let result = audio::engine_cmd_audio_source_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceDispose(result),
            });
        }
        EngineCmd::CmdAudioResourceDispose(args) => {
            let result = audio::engine_cmd_audio_resource_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioResourceDispose(result),
            });
        }
        EngineCmd::CmdGeometryUpsert(args) => {
            let result = match args {
                CmdGeometryUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_geometry_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdGeometryUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_geometry_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GeometryUpsert(result),
            });
        }
        EngineCmd::CmdGeometryDispose(args) => {
            let result = res::engine_cmd_geometry_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GeometryDispose(result),
            });
        }
        EngineCmd::CmdPrimitiveGeometryCreate(args) => {
            let result = res::engine_cmd_primitive_geometry_create(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::PrimitiveGeometryCreate(result),
            });
        }
        EngineCmd::CmdEnvironmentUpsert(args) => {
            let result = match args {
                CmdEnvironmentUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_environment_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdEnvironmentUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_environment_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::EnvironmentUpsert(result),
            });
        }
        EngineCmd::CmdEnvironmentDispose(args) => {
            let result = res::engine_cmd_environment_dispose(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::EnvironmentDispose(result),
            });
        }
        EngineCmd::CmdShadowConfigure(args) => {
            let result = res::shadow::engine_cmd_shadow_configure(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::ShadowConfigure(result),
            });
        }
        EngineCmd::CmdRealmCreate(args) => {
            let result = realm::engine_cmd_realm_create(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RealmCreate(result),
            });
        }
        EngineCmd::CmdRealmDispose(args) => {
            let result = realm::engine_cmd_realm_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RealmDispose(result),
            });
        }
        EngineCmd::CmdTargetUpsert(args) => {
            let result = target::engine_cmd_target_upsert(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetUpsert(result),
            });
        }
        EngineCmd::CmdTargetDispose(args) => {
            let result = target::engine_cmd_target_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetDispose(result),
            });
        }
        EngineCmd::CmdTargetLayerUpsert(args) => {
            let result = target::engine_cmd_target_layer_upsert(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetLayerUpsert(result),
            });
        }
        EngineCmd::CmdTargetLayerDispose(args) => {
            let result = target::engine_cmd_target_layer_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetLayerDispose(result),
            });
        }
        EngineCmd::CmdUiThemeDefine(args) => {
            super::dispatch_ui::cmd_ui_theme_define(engine, pack.id, args);
        }
        EngineCmd::CmdUiThemeDispose(args) => {
            super::dispatch_ui::cmd_ui_theme_dispose(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentCreate(args) => {
            super::dispatch_ui::cmd_ui_document_create(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentDispose(args) => {
            super::dispatch_ui::cmd_ui_document_dispose(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentSetRect(args) => {
            super::dispatch_ui::cmd_ui_document_set_rect(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentSetTheme(args) => {
            super::dispatch_ui::cmd_ui_document_set_theme(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentGetTree(args) => {
            super::dispatch_ui::cmd_ui_document_get_tree(engine, pack.id, args);
        }
        EngineCmd::CmdUiDocumentGetLayoutRects(args) => {
            super::dispatch_ui::cmd_ui_document_get_layout_rects(engine, pack.id, args);
        }
        EngineCmd::CmdUiApplyOps(args) => {
            super::dispatch_ui::cmd_ui_apply_ops(engine, pack.id, args);
        }
        EngineCmd::CmdUiDebugSet(args) => {
            super::dispatch_ui::cmd_ui_debug_set(engine, pack.id, args);
        }
        EngineCmd::CmdUiFocusSet(args) => {
            super::dispatch_ui::cmd_ui_focus_set(engine, pack.id, args);
        }
        EngineCmd::CmdUiFocusGet(args) => {
            super::dispatch_ui::cmd_ui_focus_get(engine, pack.id, args);
        }
        EngineCmd::CmdUiEventTraceSet(args) => {
            super::dispatch_ui::cmd_ui_event_trace_set(engine, pack.id, args);
        }
        EngineCmd::CmdUiImageCreateFromBuffer(args) => {
            super::dispatch_ui::cmd_ui_image_create_from_buffer(engine, pack.id, args);
        }
        EngineCmd::CmdUiImageDispose(args) => {
            super::dispatch_ui::cmd_ui_image_dispose(engine, pack.id, args);
        }
        EngineCmd::CmdUiClipboardPaste(args) => {
            super::dispatch_ui::cmd_ui_clipboard_paste(engine, pack.id, args);
        }
        EngineCmd::CmdUiScreenshotReply(args) => {
            super::dispatch_ui::cmd_ui_screenshot_reply(engine, pack.id, args);
        }
        EngineCmd::CmdUiAccessKitActionRequest(args) => {
            super::dispatch_ui::cmd_ui_accesskit_action_request(engine, pack.id, args);
        }
        EngineCmd::CmdModelList(args) => {
            let result = res::engine_cmd_model_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::ModelList(result),
            });
        }
        EngineCmd::CmdMaterialList(args) => {
            let result = res::engine_cmd_material_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialList(result),
            });
        }
        EngineCmd::CmdTextureList(args) => {
            let result = res::engine_cmd_texture_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureList(result),
            });
        }
        EngineCmd::CmdGeometryList(args) => {
            let result = res::engine_cmd_geometry_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GeometryList(result),
            });
        }
        EngineCmd::CmdLightList(args) => {
            let result = res::engine_cmd_light_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::LightList(result),
            });
        }
        EngineCmd::CmdCameraList(args) => {
            let result = res::engine_cmd_camera_list(engine, &args);
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::CameraList(result),
            });
        }
        EngineCmd::CmdGizmoDrawLine(args) => {
            for (window_id, render_state) in engine.render.states.iter_mut() {
                render_state
                    .gizmos
                    .add_line(args.start, args.end, args.color);
                if let Some(window_state) = engine.window.states.get_mut(window_id) {
                    window_state.is_dirty = true;
                }
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GizmoDrawLine(gizmo::CmdResultGizmoDraw { status: 0 }),
            });
        }
        EngineCmd::CmdGizmoDrawAabb(args) => {
            for (window_id, render_state) in engine.render.states.iter_mut() {
                render_state.gizmos.add_aabb(args.min, args.max, args.color);
                if let Some(window_state) = engine.window.states.get_mut(window_id) {
                    window_state.is_dirty = true;
                }
            }
            engine.response_queue.push(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GizmoDrawAabb(gizmo::CmdResultGizmoDraw { status: 0 }),
            });
        }
    }
}
