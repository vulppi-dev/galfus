pub mod events;
pub mod keycodes;
pub mod listeners;
mod raycast;
pub mod routing;

#[cfg(not(feature = "wasm"))]
#[allow(unused_imports)]
pub use events::{ElementState, KeyboardEvent, ModifiersState, PointerEvent, ScrollDelta};
#[cfg(not(feature = "wasm"))]
pub use vulfram_input::InputCacheManager;
#[cfg(not(feature = "wasm"))]
pub use vulfram_input::InputState;

pub use routing::route_pointer_events;
