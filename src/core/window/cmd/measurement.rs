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

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_measurement(
    engine: &mut EngineState,
    args: &CmdWindowMeasurementArgs,
) -> CmdResultWindowMeasurement {
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

    if let Some(position) = args.position {
        let next_position = PhysicalPosition::new(position.x, position.y);
        window_state.window.set_outer_position(next_position);
    }

    if let Some(size) = args.size {
        let next_size = PhysicalSize::new(size.x, size.y);
        let _ = window_state.window.request_inner_size(next_size);
        window_state.config.width = size.x;
        window_state.config.height = size.y;
        if let Some(device) = engine.device.as_ref() {
            window_state.surface.configure(device, &window_state.config);
        }
        window_state.is_dirty = true;
    }

    let wants_position = args.get_position || args.position.is_some();
    let wants_size = args.get_size || args.size.is_some();

    let current_size = if wants_size || args.get_outer_size {
        let size = window_state.window.inner_size();
        Some(UVec2::new(size.width, size.height))
    } else {
        None
    };

    let current_position = if wants_position {
        window_state
            .window
            .outer_position()
            .ok()
            .map(|p| IVec2::new(p.x, p.y))
    } else {
        None
    };

    let current_outer_size = if args.get_outer_size {
        let size = window_state.window.outer_size();
        Some(UVec2::new(size.width, size.height))
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
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_measurement(
    _engine: &mut EngineState,
    args: &CmdWindowMeasurementArgs,
) -> CmdResultWindowMeasurement {
    CmdResultWindowMeasurement {
        success: true,
        message: format!(
            "Window measurement is deferred/empty in wasm mode (window_id={})",
            args.window_id
        ),
        ..Default::default()
    }
}
