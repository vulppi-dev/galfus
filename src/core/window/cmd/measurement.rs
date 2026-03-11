#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize};
use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowMeasurementArgs {
    pub window_id: u32,
    pub position: Option<IVec2>,
    pub size: Option<UVec2>,
    pub get_position: bool,
    pub get_size: bool,
    pub get_outer_size: bool,
    pub get_surface_size: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowMeasurement {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub position: Option<IVec2>,
    #[serde(default)]
    pub size: Option<UVec2>,
    #[serde(default)]
    pub outer_size: Option<UVec2>,
    #[serde(default)]
    pub surface_size: Option<UVec2>,
}

pub fn engine_cmd_window_measurement(
    engine: &mut EngineState,
    args: &CmdWindowMeasurementArgs,
) -> CmdResultWindowMeasurement {
    let mut resized_surface_size: Option<UVec2> = None;
    let result = {
        let Some(window_state) = engine.window.states.get_mut(&args.window_id) else {
            return CmdResultWindowMeasurement {
                success: true,
                message: format!(
                    "Window {} not ready yet; returning empty measurement",
                    args.window_id
                ),
                ..Default::default()
            };
        };

        #[cfg(not(feature = "wasm"))]
        if let Some(position) = args.position {
            let next_position = PhysicalPosition::new(position.x, position.y);
            window_state.window.set_outer_position(next_position);
        }

        if let Some(size) = args.size {
            #[cfg(not(feature = "wasm"))]
            {
                let next_size = PhysicalSize::new(size.x, size.y);
                let _ = window_state.window.request_inner_size(next_size);
            }
            window_state.config.width = size.x.max(1);
            window_state.config.height = size.y.max(1);
            if let Some(device) = engine.device.as_ref() {
                window_state.surface.configure(device, &window_state.config);
                #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
                if let Some(render_state) = engine.render.get_mut(&args.window_id) {
                    render_state.on_resize(
                        device,
                        window_state.config.width,
                        window_state.config.height,
                    );
                }
                crate::core::resources::ensure_render_target(
                    device,
                    &mut window_state.surface_target,
                    window_state.config.width,
                    window_state.config.height,
                    wgpu::TextureFormat::Rgba16Float,
                );
            }
            window_state.inner_size =
                UVec2::new(window_state.config.width, window_state.config.height);
            #[cfg(feature = "wasm")]
            {
                window_state.outer_size = window_state.inner_size;
            }
            window_state.is_dirty = true;
            resized_surface_size = Some(window_state.inner_size);
        }

        let wants_position = args.get_position || args.position.is_some();
        let wants_size = args.get_size || args.size.is_some();

        let current_size = if wants_size || args.get_outer_size {
            #[cfg(not(feature = "wasm"))]
            let value = {
                let size = window_state.window.inner_size();
                let measured = UVec2::new(size.width, size.height);
                window_state.inner_size = measured;
                measured
            };
            #[cfg(feature = "wasm")]
            let value = window_state.inner_size;
            Some(value)
        } else {
            None
        };

        let current_position = if wants_position {
            #[cfg(not(feature = "wasm"))]
            {
                window_state
                    .window
                    .outer_position()
                    .ok()
                    .map(|p| IVec2::new(p.x, p.y))
            }
            #[cfg(feature = "wasm")]
            {
                None
            }
        } else {
            None
        };

        let current_outer_size = if args.get_outer_size {
            #[cfg(not(feature = "wasm"))]
            let value = {
                let size = window_state.window.outer_size();
                let measured = UVec2::new(size.width, size.height);
                window_state.outer_size = measured;
                measured
            };
            #[cfg(feature = "wasm")]
            let value = window_state.outer_size;
            Some(value)
        } else {
            None
        };

        let current_surface_size = if args.get_surface_size {
            Some(UVec2::new(
                window_state.config.width,
                window_state.config.height,
            ))
        } else {
            None
        };

        CmdResultWindowMeasurement {
            success: true,
            message: "Window measurement command applied successfully".into(),
            position: current_position,
            size: current_size,
            outer_size: current_outer_size,
            surface_size: current_surface_size,
        }
    };
    if let Some(size) = resized_surface_size {
        sync_window_surface_size(engine, args.window_id, size);
    }
    result
}

fn sync_window_surface_size(engine: &mut EngineState, window_id: u32, size: UVec2) {
    let surface_id = engine
        .universal_state
        .presents
        .entries
        .values()
        .find(|present| present.value.window_id == window_id)
        .map(|present| present.value.surface);
    if let Some(surface_id) = surface_id
        && let Some(surface_entry) = engine.universal_state.surfaces.entries.get_mut(&surface_id)
    {
        surface_entry.value.size = size;
    }
}
