#[cfg(all(target_arch = "wasm32", target_arch = "wasm32"))]
pub use web_time::Instant;

#[cfg(not(all(target_arch = "wasm32", target_arch = "wasm32")))]
pub use std::time::Instant;
