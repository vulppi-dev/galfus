#[cfg(not(target_arch = "wasm32"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::core::state::EngineState;
use glam::IVec2;
use glam::UVec2;
pub use vulfram_protocol::{CmdResultWindowMeasurement, CmdWindowMeasurementArgs};

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

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(position) = args.position {
            let next_position = PhysicalPosition::new(position.x, position.y);
            window_state.window.set_outer_position(next_position);
        }

        if let Some(size) = args.size {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let next_size = PhysicalSize::new(size.x, size.y);
                let _ = window_state.window.request_inner_size(next_size);
            }
            window_state.config.width = size.x.max(1);
            window_state.config.height = size.y.max(1);
            if let Some(device) = engine.device.as_ref() {
                window_state.surface.configure(device, &window_state.config);
                #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
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
            #[cfg(target_arch = "wasm32")]
            {
                window_state.outer_size = window_state.inner_size;
            }
            window_state.is_dirty = true;
            resized_surface_size = Some(window_state.inner_size);
        }

        let wants_position = args.get_position || args.position.is_some();
        let wants_size = args.get_size || args.size.is_some();

        let current_size = if wants_size || args.get_outer_size {
            #[cfg(not(target_arch = "wasm32"))]
            let value = {
                let size = window_state.window.inner_size();
                let measured = UVec2::new(size.width, size.height);
                window_state.inner_size = measured;
                measured
            };
            #[cfg(target_arch = "wasm32")]
            let value = window_state.inner_size;
            Some(value)
        } else {
            None
        };

        let current_position = if wants_position {
            #[cfg(not(target_arch = "wasm32"))]
            {
                window_state
                    .window
                    .outer_position()
                    .ok()
                    .map(|p| IVec2::new(p.x, p.y))
            }
            #[cfg(target_arch = "wasm32")]
            {
                None
            }
        } else {
            None
        };

        let current_outer_size = if args.get_outer_size {
            #[cfg(not(target_arch = "wasm32"))]
            let value = {
                let size = window_state.window.outer_size();
                let measured = UVec2::new(size.width, size.height);
                window_state.outer_size = measured;
                measured
            };
            #[cfg(target_arch = "wasm32")]
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
        .composition
        .presents
        .entries
        .values()
        .find(|present| present.value.window_id == window_id)
        .map(|present| present.value.surface);
    if let Some(surface_id) = surface_id
        && let Some(surface_entry) = engine
            .universal_state
            .composition
            .surfaces
            .entries
            .get_mut(&surface_id)
    {
        surface_entry.value.size = size;
    }
}

#[cfg_attr(
    not(all(target_arch = "wasm32", target_arch = "wasm32")),
    allow(dead_code)
)]
pub fn resolve_canvas_surface_size_pixels(
    attr_width: u32,
    attr_height: u32,
    css_width: f64,
    css_height: f64,
    dpr: f64,
) -> UVec2 {
    let safe_dpr = dpr.max(1.0);
    let css_pixels = UVec2::new(
        (css_width * safe_dpr).round().max(1.0) as u32,
        (css_height * safe_dpr).round().max(1.0) as u32,
    );
    let attr_pixels = UVec2::new(attr_width.max(1), attr_height.max(1));
    let using_default_attrs = attr_pixels.x == 300 && attr_pixels.y == 150;
    if using_default_attrs {
        css_pixels
    } else {
        attr_pixels
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_canvas_surface_size_pixels;
    use glam::UVec2;

    #[test]
    fn defaults_to_css_times_dpr_when_canvas_attrs_are_default() {
        let size = resolve_canvas_surface_size_pixels(300, 150, 640.0, 360.0, 2.0);
        assert_eq!(size, UVec2::new(1280, 720));
    }

    #[test]
    fn respects_explicit_canvas_drawing_buffer_size() {
        let size = resolve_canvas_surface_size_pixels(1024, 576, 640.0, 360.0, 2.0);
        assert_eq!(size, UVec2::new(1024, 576));
    }
}
