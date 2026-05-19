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
