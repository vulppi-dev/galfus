use crate::core::ui::UiState;

#[derive(Debug, Default)]
pub struct InteractionRuntimeState {
    pub ui: UiState,
    pub input_routing: galfus_input::InputRoutingState,
}
