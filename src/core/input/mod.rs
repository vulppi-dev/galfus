#[cfg(not(feature = "wasm"))]
pub mod cache;
pub mod events;
pub mod keycodes;
pub mod listeners;
mod raycast;
pub mod routing;
#[cfg(not(feature = "wasm"))]
pub mod state;

#[cfg(not(feature = "wasm"))]
pub use cache::InputCacheManager;
#[cfg(not(feature = "wasm"))]
#[allow(unused_imports)]
pub use events::{ElementState, KeyboardEvent, ModifiersState, PointerEvent, ScrollDelta};
#[cfg(not(feature = "wasm"))]
pub use state::InputState;

pub use routing::route_pointer_events;
