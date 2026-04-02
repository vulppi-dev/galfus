#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use web_time::Instant;

#[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
pub use std::time::Instant;
