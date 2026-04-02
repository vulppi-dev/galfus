#[cfg(not(feature = "wasm"))]
pub use vulfram_platform::{
    map_gilrs_axis as convert_gilrs_axis, map_gilrs_button as convert_gilrs_button,
};
