use std::sync::Arc;

use super::create_shared::{register_window_realm, resolve_rgba16f_msaa_supported_mask};
use super::{CmdResultWindowCreate, CmdWindowCreateArgs};
use crate::core::platform::ActiveEventLoop;
use crate::core::platform::Window;
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize, Position};
use crate::core::profiling::gpu::GpuProfiler;
use crate::core::state::EngineState;
use crate::core::window::WindowState;
use glam::{IVec2, UVec2};
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
    let mut gpu_profiling_supported = false;
    let (adapter, is_new_device) = if engine.device.is_none() {
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

        let adapter_features = adapter.features();
        let mut required_features = wgpu::Features::empty();
        gpu_profiling_supported = adapter_features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
        if gpu_profiling_supported {
            required_features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }
        let adapter_specific_format_features_supported = adapter_features
            .contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
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
        engine.rgba16f_msaa_supported_mask = resolve_rgba16f_msaa_supported_mask(
            &adapter,
            adapter_specific_format_features_supported,
        );

        engine.caps = Some(surface.get_capabilities(&adapter));
        engine.device = Some(device);
        engine.queue = Some(queue);
        (adapter, true)
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
        (adapter, false)
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

    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0]);

    // Use Opaque alpha mode to prevent window transparency
    // This ensures the clear color is rendered correctly
    let alpha_mode = wgpu::CompositeAlphaMode::Auto;

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        width: window_width,
        height: window_height,
        present_mode: if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        },
        format,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    // Configure the surface with the device
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
    surface.configure(device, &config);

    // Get initial window positions and sizes
    let inner_position = window.inner_position().unwrap_or_default();
    let outer_position = window.outer_position().unwrap_or_default();
    let inner_size = window.inner_size();
    let outer_size = window.outer_size();

    // Create render state and initialize blit resources
    let mut render_state = crate::core::render::RenderState::new(format);
    render_state.rgba16f_msaa_supported_mask = engine.rgba16f_msaa_supported_mask;
    let mut surface_target = None;
    if let Some(device) = &engine.device {
        if let Some(queue) = &engine.queue {
            render_state.init(device, queue, format);

            // Initialize size-dependent resources (like depth buffer)
            render_state.on_resize(device, window_width, window_height);
            crate::core::resources::ensure_render_target(
                device,
                &mut surface_target,
                window_width,
                window_height,
                wgpu::TextureFormat::Rgba16Float,
            );
        }
    }

    engine.window.insert_state(
        win_id,
        WindowState {
            window,
            surface,
            config: config.clone(),
            inner_position: IVec2::new(inner_position.x, inner_position.y),
            outer_position: IVec2::new(outer_position.x, outer_position.y),
            inner_size: UVec2::new(inner_size.width, inner_size.height),
            outer_size: UVec2::new(outer_size.width, outer_size.height),
            render_state,
            surface_target,
            is_dirty: true,
            last_present_instant: None,
            last_frame_delta_ns: 0,
            fps_instant: 0.0,
        },
    );
    let binding = register_window_realm(engine, win_id, UVec2::new(window_width, window_height));

    if is_new_device && gpu_profiling_supported && engine.gpu_profiler.is_none() {
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
