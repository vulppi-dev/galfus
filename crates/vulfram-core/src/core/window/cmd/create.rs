#[path = "create_native.rs"]
mod create_native;
#[path = "create_shared.rs"]
mod create_shared;
#[path = "create_wasm.rs"]
mod create_wasm;

// MARK: - Create Window

pub use vulfram_protocol::{CmdResultWindowCreate, CmdWindowCreateArgs};

#[cfg(not(feature = "wasm"))]
pub use create_native::engine_cmd_window_create;
#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
pub use create_wasm::engine_cmd_window_create;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use create_wasm::engine_cmd_window_create_async;
