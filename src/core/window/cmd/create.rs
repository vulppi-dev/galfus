use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

use super::{EngineWindowState, window_size_default};

mod create_native;
mod create_shared;
mod create_wasm;

// MARK: - Create Window

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub window_id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: UVec2,
    #[serde(default)]
    pub position: IVec2,
    #[serde(default)]
    pub canvas_id: Option<String>,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default)]
    pub resizable: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub realm_id: Option<u32>,
    #[serde(default)]
    pub surface_id: Option<u32>,
    #[serde(default)]
    pub present_id: Option<u32>,
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use create_wasm::engine_cmd_window_create_async;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use create_wasm::engine_cmd_window_create;
#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
pub use create_wasm::engine_cmd_window_create;
#[cfg(not(feature = "wasm"))]
pub use create_native::engine_cmd_window_create;
