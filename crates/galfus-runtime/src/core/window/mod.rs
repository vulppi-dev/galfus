#[cfg(not(target_arch = "wasm32"))]
mod cache;
mod cmd;
mod state;

pub use cmd::*;
pub use galfus_protocol::{WindowCanvasActiveState, WindowEvent, WindowPointerCaptureState};
#[cfg(target_arch = "wasm32")]
pub use state::WebListenerRegistration;
pub use state::WindowManager;
pub use state::WindowState;
