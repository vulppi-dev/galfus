#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineEvent};
#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
use crate::core::platform::ActiveEventLoop;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::singleton::with_engine_singleton;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::window::WindowEvent;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen::JsCast;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen_futures::spawn_local;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use web_sys::HtmlCanvasElement;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use super::create_shared::{register_window_realm, resolve_rgba16f_msaa_supported_mask};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::platform::Window;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::profiling::gpu::GpuProfiler;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::window::WindowState;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use glam::UVec2;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
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
    let bootstrap_target = vulfram_platform::PlatformRenderBootstrapTarget::new(
        args.window_id,
        window_size,
        vulfram_platform::PlatformRenderSurfaceKind::WebCanvas,
        vulfram_platform::PlatformSurfaceAlphaMode::Opaque,
        false,
    );

    let win_id = args.window_id;
    let canvas_clone = canvas.clone();
    spawn_local(async move {
        let bootstrap_plan = vulfram_runtime::plan_render_bootstrap(false, bootstrap_target);
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface =
            match instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas_clone.clone())) {
                Ok(surface) => surface,
                Err(e) => {
                    let _ = with_engine_singleton(|engine| {
                        engine
                            .state
                            .runtime
                            .response_queue
                            .push(CommandResponseEnvelope {
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
                    engine
                        .state
                        .runtime
                        .response_queue
                        .push(CommandResponseEnvelope {
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

        let adapter_features = adapter.features();
        let mut required_features = wgpu::Features::empty();
        let gpu_profiling_supported = adapter_features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
        if gpu_profiling_supported {
            required_features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }

        if adapter_features.contains(wgpu::Features::POLYGON_MODE_LINE) {
            required_features |= wgpu::Features::POLYGON_MODE_LINE;
        }
        if adapter_features.contains(wgpu::Features::POLYGON_MODE_POINT) {
            required_features |= wgpu::Features::POLYGON_MODE_POINT;
        }
        let adapter_specific_format_features_supported =
            adapter_features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
        if adapter_specific_format_features_supported {
            required_features |= wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        }

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                let _ = with_engine_singleton(|engine| {
                    engine
                        .state
                        .runtime
                        .response_queue
                        .push(CommandResponseEnvelope {
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
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: bootstrap_plan.target.size.x,
            height: bootstrap_plan.target.size.y,
            present_mode: wgpu::PresentMode::Fifo,
            format,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut render_state = crate::core::render::RenderState::new(format);
        let rgba16f_msaa_supported_mask = resolve_rgba16f_msaa_supported_mask(
            &adapter,
            adapter_specific_format_features_supported,
        );
        render_state.rgba16f_msaa_supported_mask = rgba16f_msaa_supported_mask;
        render_state.init(&device, &queue, format);
        render_state.on_resize(
            &device,
            bootstrap_plan.target.size.x,
            bootstrap_plan.target.size.y,
        );
        let mut surface_target = None;
        crate::core::resources::ensure_render_target(
            &device,
            &mut surface_target,
            bootstrap_plan.target.size.x,
            bootstrap_plan.target.size.y,
            wgpu::TextureFormat::Rgba16Float,
        );

        let listeners =
            crate::core::platforms::browser::input::attach_canvas_listeners(win_id, &canvas_clone);
        let window_handle = std::sync::Arc::new(Window::new(win_id, canvas_clone.clone()));

        let _ = with_engine_singleton(|engine| {
            engine.state.wgpu = instance;
            engine.state.caps = Some(caps);
            engine.state.rgba16f_msaa_supported_mask = rgba16f_msaa_supported_mask;
            engine.state.device = Some(device);
            engine.state.queue = Some(queue);
            engine.state.window.map_window(window_handle.id(), win_id);
            engine.state.render.insert(win_id, render_state);
            engine.state.window.insert_state(
                win_id,
                WindowState {
                    window: window_handle,
                    surface,
                    config: config.clone(),
                    inner_size: bootstrap_plan.target.size,
                    outer_size: bootstrap_plan.target.size,
                    surface_target,
                    is_dirty: true,
                    last_present_ns: 0,
                    last_frame_delta_ns: 0,
                    fps_instant: 0.0,
                    web_listener_registrations: listeners,
                },
            );
            engine
                .state
                .window
                .set_cursor_grab_mode(win_id, crate::core::window::CursorGrabMode::None);
            engine
                .state
                .window
                .set_pointer_capture_active(win_id, false);
            engine
                .state
                .window
                .set_lifecycle_state(win_id, crate::core::window::EngineWindowState::Windowed);
            let binding =
                register_window_realm(&mut engine.state, win_id, bootstrap_plan.target.size);

            engine
                .state
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnCreate {
                    window_id: win_id,
                }));
            engine
                .state
                .runtime
                .response_queue
                .push(CommandResponseEnvelope {
                    id: cmd_id,
                    response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                        success: true,
                        message: "Canvas window created successfully".into(),
                        realm_id: Some(binding.realm_id.0),
                        surface_id: Some(binding.surface_id.0),
                        present_id: Some(binding.present_id.0),
                    }),
                });
            if gpu_profiling_supported && engine.state.gpu_profiler.is_none() {
                if let (Some(device), Some(queue)) =
                    (engine.state.device.as_ref(), engine.state.queue.as_ref())
                {
                    engine.state.gpu_profiler = Some(GpuProfiler::new(
                        device,
                        queue,
                        engine.state.window.states.len(),
                    ));
                }
            }
        });
    });

    Ok(())
}

#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
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
