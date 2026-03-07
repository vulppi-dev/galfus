#[cfg(not(feature = "wasm"))]
pub mod cache;
pub mod events;
pub mod keycodes;
pub mod listeners;
mod raycast;
pub mod routing;
#[cfg(not(feature = "wasm"))]
pub mod state;
#[cfg(test)]
mod tests_phase10;

#[cfg(not(feature = "wasm"))]
pub use cache::InputCacheManager;
#[cfg(not(feature = "wasm"))]
pub use events::{ElementState, KeyboardEvent, ModifiersState, PointerEvent, ScrollDelta};
#[cfg(not(feature = "wasm"))]
pub use events::{
    convert_key_code, convert_key_location, convert_mouse_button, convert_touch_phase,
};
#[cfg(not(feature = "wasm"))]
pub use state::InputState;

pub use routing::route_pointer_events;
