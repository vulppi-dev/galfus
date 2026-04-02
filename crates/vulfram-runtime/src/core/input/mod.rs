pub mod events;
pub mod keycodes;
pub mod listeners;
mod raycast;
pub mod routing;
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
pub use events::{ElementState, KeyboardEvent, ModifiersState, PointerEvent, ScrollDelta};
#[cfg(not(target_arch = "wasm32"))]
pub use vulfram_input::InputCacheManager;
#[cfg(not(target_arch = "wasm32"))]
pub use vulfram_input::InputState;

pub use routing::route_pointer_events;
pub use state::InteractionRuntimeState;
