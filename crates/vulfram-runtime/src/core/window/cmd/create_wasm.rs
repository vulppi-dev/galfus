#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineEvent};
#[cfg(all(target_arch = "wasm32", not(target_arch = "wasm32")))]
use crate::core::platform::ActiveEventLoop;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::singleton::with_engine_singleton;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::window::WindowEvent;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use wasm_bindgen::JsCast;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use wasm_bindgen_futures::spawn_local;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use web_sys::HtmlCanvasElement;

#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use super::create_shared::{build_window_render_bootstrap_artifacts, register_window_realm};
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(all(target_arch = "wasm32", not(target_arch = "wasm32")))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::platform::Window;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::profiling::gpu::GpuProfiler;
#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
use crate::core::window::WindowState;

#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
pub fn engine_cmd_window_create_async(
    args: &CmdWindowCreateArgs,
    cmd_id: u64,
) -> Result<(), CmdResultWindowCreate> {
    let canvas_id = match &args.canvas_id {
        Some(id) => id,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "canvasId is required in wasm mode".into(),
                realm_id: None,
                surface_id: None,
                present_id: None,
            });
        }
    };

    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "Web window not available".into(),
                realm_id: None,
                surface_id: None,
                present_id: None,
            });
        }
    };
    let document = match window.document() {
        Some(document) => document,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "Document not available".into(),
                realm_id: None,
                surface_id: None,
                present_id: None,
            });
        }
    };
    let element = match document.get_element_by_id(canvas_id) {
        Some(element) => element,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: format!("Canvas with id '{}' not found", canvas_id),
                realm_id: None,
                surface_id: None,
                present_id: None,
            });
        }
    };
    let canvas: HtmlCanvasElement = match element.dyn_into() {
        Ok(canvas) => canvas,
        Err(_) => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: format!("Element '{}' is not a canvas", canvas_id),
                realm_id: None,
                surface_id: None,
                present_id: None,
            });
        }
    };

    let rect = canvas.get_bounding_client_rect();
    let dpr = window.device_pixel_ratio();
    let window_size = crate::core::window::cmd::resolve_canvas_surface_size_pixels(
        canvas.width(),
        canvas.height(),
        rect.width(),
        rect.height(),
        dpr,
    );
    let bootstrap_target =
        vulfram_platform::plan_web_render_bootstrap_target(args.window_id, window_size);

    let win_id = args.window_id;
    let canvas_clone = canvas.clone();
    spawn_local(async move {
        let bootstrap_plan = vulfram_runtime::plan_render_bootstrap(false, bootstrap_target);
        let instance = vulfram_render::create_default_instance();
        let surface =
            match instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas_clone.clone())) {
                Ok(surface) => surface,
                Err(e) => {
                    let _ = with_engine_singleton(|engine| {
                        engine.state.runtime.push_response(CommandResponseEnvelope {
                            id: cmd_id,
                            response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                                success: false,
                                message: format!("WGPU create surface error: {}", e),
                                realm_id: None,
                                surface_id: None,
                                present_id: None,
                            }),
                        });
                    });
                    return;
                }
            };

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(_) => {
                let _ = with_engine_singleton(|engine| {
                    engine.state.runtime.push_response(CommandResponseEnvelope {
                        id: cmd_id,
                        response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                            success: false,
                            message: "WGPU adapter request error".into(),
                            realm_id: None,
                            surface_id: None,
                            present_id: None,
                        }),
                    });
                });
                return;
            }
        };

        let adapter_info = vulfram_render::analyze_adapter(&adapter);

        let (device, queue) = match adapter
            .request_device(&vulfram_render::build_device_descriptor(
                adapter_info.feature_plan,
            ))
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                let _ = with_engine_singleton(|engine| {
                    engine.state.runtime.push_response(CommandResponseEnvelope {
                        id: cmd_id,
                        response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                            success: false,
                            message: format!("WGPU device request error: {}", e),
                            realm_id: None,
                            surface_id: None,
                            present_id: None,
                        }),
                    });
                });
                return;
            }
        };

        let caps = surface.get_capabilities(&adapter);
        let artifacts = build_window_render_bootstrap_artifacts(
            &surface,
            &device,
            &queue,
            &caps,
            bootstrap_plan.target,
            adapter_info.rgba16f_msaa_supported_mask,
        );

        let listeners =
            crate::core::platforms::browser::input::attach_canvas_listeners(win_id, &canvas_clone);
        let window_handle = std::sync::Arc::new(Window::new(win_id, canvas_clone.clone()));

        let _ = with_engine_singleton(|engine| {
            engine.state.wgpu = instance;
            engine.state.caps = Some(caps);
            engine.state.rgba16f_msaa_supported_mask = adapter_info.rgba16f_msaa_supported_mask;
            engine.state.device = Some(device);
            engine.state.queue = Some(queue);
            engine.state.window.map_window(window_handle.id(), win_id);
            engine.state.render.insert(win_id, artifacts.render_state);
            engine.state.window.insert_state(
                win_id,
                WindowState::new_web(
                    window_handle,
                    surface,
                    artifacts.config.clone(),
                    bootstrap_plan.target.size,
                    artifacts.surface_target,
                    listeners,
                ),
            );
            engine.state.window.initialize_window_defaults(win_id);
            let binding =
                register_window_realm(&mut engine.state, win_id, bootstrap_plan.target.size);

            engine
                .state
                .runtime
                .push_event(EngineEvent::Window(WindowEvent::OnCreate {
                    window_id: win_id,
                }));
            engine.state.runtime.push_response(CommandResponseEnvelope {
                id: cmd_id,
                response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                    success: true,
                    message: "Canvas window created successfully".into(),
                    realm_id: Some(binding.realm_id.0),
                    surface_id: Some(binding.surface_id.0),
                    present_id: Some(binding.present_id.0),
                }),
            });
            {
                if let (Some(device), Some(queue)) =
                    (engine.state.device.as_ref(), engine.state.queue.as_ref())
                {
                    GpuProfiler::ensure_available(
                        &mut engine.state.gpu_profiler,
                        device,
                        queue,
                        engine.state.window.states.len(),
                        adapter_info.feature_plan.gpu_profiling_supported,
                    );
                }
            }
        });
    });

    Ok(())
}

#[cfg(all(target_arch = "wasm32", not(target_arch = "wasm32")))]
pub fn engine_cmd_window_create(
    _engine: &mut crate::core::state::EngineState,
    _event_loop: &ActiveEventLoop,
    _args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
    CmdResultWindowCreate {
        success: false,
        message: "wasm feature requires the wasm32-unknown-unknown target".into(),
        realm_id: None,
        surface_id: None,
        present_id: None,
    }
}
