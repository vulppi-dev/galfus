#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use super::create_shared::{build_window_render_bootstrap_artifacts, register_window_realm};
#[cfg(not(target_arch = "wasm32"))]
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
#[cfg(not(target_arch = "wasm32"))]
use crate::core::platform::ActiveEventLoop;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::platform::Window;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize, Position};
#[cfg(not(target_arch = "wasm32"))]
use crate::core::profiling::gpu::GpuProfiler;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::state::EngineState;
use crate::core::id_policy::validate_host_logical_id;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::window::WindowState;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::window::apply_window_state_to_window;
#[cfg(not(target_arch = "wasm32"))]
use glam::{IVec2, UVec2};
#[cfg(not(target_arch = "wasm32"))]
use pollster::FutureExt;

#[cfg(not(target_arch = "wasm32"))]
pub fn engine_cmd_window_create(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
    if let Err(message) = validate_host_logical_id(args.window_id, "windowId") {
        return CmdResultWindowCreate {
            success: false,
            message,
            realm_id: None,
        };
    }
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
            return CmdResultWindowCreate {
                success: false,
                message: format!("Winit create window error: {}", e),
                realm_id: None,
            };
        }
    };

    if let Err(message) = apply_window_state_to_window(&window, args.initial_state) {
        return CmdResultWindowCreate {
            success: false,
            message,
            realm_id: None,
        };
    }

    let win_id = args.window_id;
    engine.window.map_window(window.id(), win_id);

    let initial_inner_size = window.inner_size();

    let bootstrap_target = galfus_platform::plan_native_render_bootstrap_target(
        args.window_id,
        UVec2::new(
            initial_inner_size.width.max(1),
            initial_inner_size.height.max(1),
        ),
        args.transparent,
    );
    let bootstrap_plan =
        galfus_runtime::plan_render_bootstrap(engine.device.is_some(), bootstrap_target);

    let surface = match engine.wgpu.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            return CmdResultWindowCreate {
                success: false,
                message: format!("WGPU create surface error: {}", e),
                realm_id: None,
            };
        }
    };

    // Get or create adapter and device
    let (adapter, is_new_device, adapter_info) = if matches!(
        bootstrap_plan.device_strategy,
        galfus_runtime::RenderBootstrapDeviceStrategy::CreateSharedDevice
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
                    };
                }
            };

        let adapter_info = galfus_render::analyze_adapter(&adapter);

        let (device, queue) = match adapter
            .request_device(&galfus_render::build_device_descriptor(
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
                };
            }
        };
        let adapter_info = galfus_render::analyze_adapter(&adapter);
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
        WindowState::new_native(
            window,
            surface,
            artifacts.config.clone(),
            IVec2::new(inner_position.x, inner_position.y),
            IVec2::new(outer_position.x, outer_position.y),
            UVec2::new(inner_size.width, inner_size.height),
            UVec2::new(outer_size.width, outer_size.height),
            artifacts.surface_target,
        ),
    );
    engine.window.initialize_window_defaults(win_id);
    let binding = register_window_realm(engine, win_id, bootstrap_plan.target.size);

    if is_new_device {
        if let (Some(device), Some(queue)) = (&engine.device, &engine.queue) {
            GpuProfiler::ensure_available(
                &mut engine.gpu_profiler,
                device,
                queue,
                engine.window.states.len(),
                adapter_info.feature_plan.gpu_profiling_supported,
            );
        }
    }

    // Initialize window cache
    engine.window.cache.initialize_window_cache(
        win_id,
        IVec2::new(inner_position.x, inner_position.y),
        IVec2::new(outer_position.x, outer_position.y),
        UVec2::new(inner_size.width, inner_size.height),
        UVec2::new(outer_size.width, outer_size.height),
    );

    CmdResultWindowCreate {
        success: true,
        message: "Window created successfully".into(),
        realm_id: Some(binding.realm_id.0),
    }
}
