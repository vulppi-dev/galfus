#[cfg(not(feature = "wasm"))]
use std::sync::Arc;

#[cfg(not(feature = "wasm"))]
use super::create_shared::{build_window_render_bootstrap_artifacts, register_window_realm};
#[cfg(not(feature = "wasm"))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(not(feature = "wasm"))]
use crate::core::platform::ActiveEventLoop;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::Window;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize, Position};
#[cfg(not(feature = "wasm"))]
use crate::core::profiling::gpu::GpuProfiler;
#[cfg(not(feature = "wasm"))]
use crate::core::state::EngineState;
#[cfg(not(feature = "wasm"))]
use crate::core::window::WindowState;
#[cfg(not(feature = "wasm"))]
use glam::{IVec2, UVec2};
#[cfg(not(feature = "wasm"))]
use pollster::FutureExt;

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_create(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
    // Ensure minimum valid size
    let window_width = args.size.x.max(100);
    let window_height = args.size.y.max(100);

    let mut win_attrs = Window::default_attributes()
        .with_title(args.title.as_str())
        .with_decorations(!args.borderless)
        .with_resizable(args.resizable)
        .with_min_inner_size(PhysicalSize::new(100, 100))
        .with_inner_size(PhysicalSize::new(window_width, window_height))
        .with_transparent(args.transparent);

    // Only set position if explicitly provided (not default 0, 0)
    // Wayland doesn't support arbitrary window positioning
    if args.position.x != 0 || args.position.y != 0 {
        win_attrs = win_attrs.with_position(Position::Physical(PhysicalPosition::new(
            args.position.x,
            args.position.y,
        )));
    }

    let window = match event_loop.create_window(win_attrs) {
        Ok(window) => Arc::new(window),
        Err(e) => {
            println!("Failed to create window: {}", e);
            return CmdResultWindowCreate {
                success: false,
                message: format!("Winit create window error: {}", e),
                realm_id: None,
                surface_id: None,
                present_id: None,
            };
        }
    };

    let win_id = args.window_id;
    engine.window.map_window(window.id(), win_id);

    let bootstrap_target = vulfram_platform::plan_native_render_bootstrap_target(
        args.window_id,
        UVec2::new(window_width, window_height),
        args.transparent,
    );
    let bootstrap_plan =
        vulfram_runtime::plan_render_bootstrap(engine.device.is_some(), bootstrap_target);

    let surface = match engine.wgpu.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            return CmdResultWindowCreate {
                success: false,
                message: format!("WGPU create surface error: {}", e),
                realm_id: None,
                surface_id: None,
                present_id: None,
            };
        }
    };

    // Get or create adapter and device
    let (adapter, is_new_device, adapter_info) = if matches!(
        bootstrap_plan.device_strategy,
        vulfram_runtime::RenderBootstrapDeviceStrategy::CreateSharedDevice
    ) {
        // First window - create new adapter and device
        let adapter =
            match pollster::block_on(engine.wgpu.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })) {
                Ok(adapter) => adapter,
                Err(_) => {
                    return CmdResultWindowCreate {
                        success: false,
                        message: "WGPU adapter request error".into(),
                        realm_id: None,
                        surface_id: None,
                        present_id: None,
                    };
                }
            };

        let adapter_info = vulfram_render::analyze_adapter(&adapter);

        let (device, queue) = match adapter
            .request_device(&vulfram_render::build_device_descriptor(
                adapter_info.feature_plan,
            ))
            .block_on()
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                return CmdResultWindowCreate {
                    success: false,
                    message: format!("WGPU device request error: {}", e),
                    realm_id: None,
                    surface_id: None,
                    present_id: None,
                };
            }
        };
        engine.rgba16f_msaa_supported_mask = adapter_info.rgba16f_msaa_supported_mask;

        engine.caps = Some(surface.get_capabilities(&adapter));
        engine.device = Some(device);
        engine.queue = Some(queue);
        (adapter, true, adapter_info)
    } else {
        // Subsequent windows - validate surface compatibility with existing adapter
        let adapter = match pollster::block_on(engine.wgpu.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )) {
            Ok(adapter) => adapter,
            Err(_) => {
                return CmdResultWindowCreate {
                    success: false,
                    message:
                        "Surface is not compatible with existing WGPU adapter. Cannot create window."
                            .into(),
                    realm_id: None,
                    surface_id: None,
                    present_id: None,
                };
            }
        };
        let adapter_info = vulfram_render::analyze_adapter(&adapter);
        (adapter, false, adapter_info)
    };

    // Get surface capabilities
    let caps = if is_new_device {
        match engine.caps.as_ref() {
            Some(caps) => caps,
            None => {
                return CmdResultWindowCreate {
                    success: false,
                    message: "Surface capabilities not initialized".into(),
                    realm_id: None,
                    surface_id: None,
                    present_id: None,
                };
            }
        }
    } else {
        // For subsequent windows, get fresh capabilities and store them
        let new_caps = surface.get_capabilities(&adapter);
        engine.caps = Some(new_caps);
        match engine.caps.as_ref() {
            Some(caps) => caps,
            None => {
                return CmdResultWindowCreate {
                    success: false,
                    message: "Surface capabilities not initialized".into(),
                    realm_id: None,
                    surface_id: None,
                    present_id: None,
                };
            }
        }
    };

    let device = match engine.device.as_ref() {
        Some(device) => device,
        None => {
            return CmdResultWindowCreate {
                success: false,
                message: "Graphics device not initialized".into(),
                realm_id: None,
                surface_id: None,
                present_id: None,
            };
        }
    };
    let queue = match engine.queue.as_ref() {
        Some(queue) => queue,
        None => {
            return CmdResultWindowCreate {
                success: false,
                message: "Graphics queue not initialized".into(),
                realm_id: None,
                surface_id: None,
                present_id: None,
            };
        }
    };
    let artifacts = build_window_render_bootstrap_artifacts(
        &surface,
        device,
        queue,
        caps,
        bootstrap_plan.target,
        adapter_info.rgba16f_msaa_supported_mask,
    );
    engine.rgba16f_msaa_supported_mask = adapter_info.rgba16f_msaa_supported_mask;

    // Get initial window positions and sizes
    let inner_position = window.inner_position().unwrap_or_default();
    let outer_position = window.outer_position().unwrap_or_default();
    let inner_size = window.inner_size();
    let outer_size = window.outer_size();

    engine.render.insert(win_id, artifacts.render_state);
    engine.window.insert_state(
        win_id,
        WindowState {
            window,
            surface,
            config: artifacts.config.clone(),
            inner_position: IVec2::new(inner_position.x, inner_position.y),
            outer_position: IVec2::new(outer_position.x, outer_position.y),
            inner_size: UVec2::new(inner_size.width, inner_size.height),
            outer_size: UVec2::new(outer_size.width, outer_size.height),
            surface_target: artifacts.surface_target,
            is_dirty: true,
            last_present_instant: None,
            last_frame_delta_ns: 0,
            fps_instant: 0.0,
            redraw_force_until_ms: 0,
        },
    );
    engine
        .window
        .set_cursor_grab_mode(win_id, crate::core::window::CursorGrabMode::None);
    engine.window.set_pointer_capture_active(win_id, false);
    engine
        .window
        .set_lifecycle_state(win_id, crate::core::window::EngineWindowState::Windowed);
    let binding = register_window_realm(engine, win_id, bootstrap_plan.target.size);

    if is_new_device
        && adapter_info.feature_plan.gpu_profiling_supported
        && engine.gpu_profiler.is_none()
    {
        if let (Some(device), Some(queue)) = (&engine.device, &engine.queue) {
            engine.gpu_profiler = Some(GpuProfiler::new(device, queue, engine.window.states.len()));
        }
    }

    // Initialize window cache
    let cache = engine.window.cache.get_or_create(win_id);
    cache.inner_position = IVec2::new(inner_position.x, inner_position.y);
    cache.outer_position = IVec2::new(outer_position.x, outer_position.y);
    cache.inner_size = UVec2::new(inner_size.width, inner_size.height);
    cache.outer_size = UVec2::new(outer_size.width, outer_size.height);
    cache.scale_factor = 1.0; // Will be updated on first scale factor change event
    cache.focused = false;
    cache.occluded = false;
    cache.dark_mode = false;

    CmdResultWindowCreate {
        success: true,
        message: "Window created successfully".into(),
        realm_id: Some(binding.realm_id.0),
        surface_id: Some(binding.surface_id.0),
        present_id: Some(binding.present_id.0),
    }
}
