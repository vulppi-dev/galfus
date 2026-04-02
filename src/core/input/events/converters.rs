#[cfg(not(feature = "wasm"))]
pub use vulfram_platform::{
    map_winit_key_location as convert_key_location, map_winit_mouse_button as convert_mouse_button,
    map_winit_physical_key_code as convert_key_code, map_winit_touch_phase as convert_touch_phase,
};
