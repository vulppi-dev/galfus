mod emit;
mod model;
mod store;

pub use emit::emit_target_listener_events;
pub use model::InputTargetListenerConfig;
pub use store::InputTargetListenerStore;

#[cfg(test)]
#[path = "listeners_tests.rs"]
mod tests;
