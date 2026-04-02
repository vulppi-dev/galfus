use crate::core::ui::UiState;

#[derive(Debug, Default)]
pub struct InteractionRuntimeState {
    pub ui: UiState,
    pub input_routing: vulfram_input::InputRoutingState,
    pub target_listeners: crate::core::input::listeners::InputTargetListenerStore,
}
