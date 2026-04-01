use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ElementState;

pub const GAMEPAD_AXIS_DEAD_ZONE: f32 = 0.1;
pub const GAMEPAD_AXIS_CHANGE_THRESHOLD: f32 = 0.01;
pub const GAMEPAD_BUTTON_CHANGE_THRESHOLD: f32 = 0.05;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum GamepadEvent {
    #[serde(rename_all = "camelCase")]
    OnConnect { gamepad_id: u32, name: String },
    #[serde(rename_all = "camelCase")]
    OnDisconnect { gamepad_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnButton {
        gamepad_id: u32,
        button: u32,
        state: ElementState,
        value: f32,
    },
    #[serde(rename_all = "camelCase")]
    OnAxis {
        gamepad_id: u32,
        axis: u32,
        value: f32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct GamepadStateCache {
    pub axes: HashMap<u32, f32>,
    pub buttons: HashMap<u32, (ElementState, f32)>,
}

impl GamepadStateCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_dead_zone(value: f32) -> f32 {
        if value.abs() < GAMEPAD_AXIS_DEAD_ZONE {
            0.0
        } else {
            let sign = value.signum();
            let adjusted = (value.abs() - GAMEPAD_AXIS_DEAD_ZONE) / (1.0 - GAMEPAD_AXIS_DEAD_ZONE);
            sign * adjusted
        }
    }

    pub fn axis_changed(&self, axis: u32, new_value: f32) -> bool {
        let adjusted_value = Self::apply_dead_zone(new_value);

        if let Some(&cached_value) = self.axes.get(&axis) {
            (cached_value - adjusted_value).abs() > GAMEPAD_AXIS_CHANGE_THRESHOLD
        } else {
            adjusted_value.abs() > GAMEPAD_AXIS_CHANGE_THRESHOLD
        }
    }

    pub fn button_changed(&self, button: u32, new_state: ElementState, new_value: f32) -> bool {
        if let Some(&(cached_state, cached_value)) = self.buttons.get(&button) {
            cached_state != new_state
                || (cached_value - new_value).abs() > GAMEPAD_BUTTON_CHANGE_THRESHOLD
        } else {
            true
        }
    }

    pub fn get_axis_value(&self, axis: u32) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }

    pub fn update_axis(&mut self, axis: u32, value: f32) {
        let adjusted_value = Self::apply_dead_zone(value);
        self.axes.insert(axis, adjusted_value);
    }

    pub fn update_axis_and_get(&mut self, axis: u32, value: f32) -> f32 {
        self.update_axis(axis, value);
        self.get_axis_value(axis)
    }

    pub fn update_button(&mut self, button: u32, state: ElementState, value: f32) {
        self.buttons.insert(button, (state, value));
    }

    pub fn button_state_from_value(value: f32) -> ElementState {
        if value > 0.5 {
            ElementState::Pressed
        } else {
            ElementState::Released
        }
    }

    pub fn update_button_event(
        &mut self,
        gamepad_id: u32,
        button: u32,
        value: f32,
    ) -> Option<GamepadEvent> {
        let state = Self::button_state_from_value(value);
        if !self.button_changed(button, state, value) {
            return None;
        }
        self.update_button(button, state, value);
        Some(GamepadEvent::OnButton {
            gamepad_id,
            button,
            state,
            value,
        })
    }

    pub fn update_axis_event(
        &mut self,
        gamepad_id: u32,
        axis: u32,
        value: f32,
    ) -> Option<GamepadEvent> {
        if !self.axis_changed(axis, value) {
            return None;
        }
        Some(GamepadEvent::OnAxis {
            gamepad_id,
            axis,
            value: self.update_axis_and_get(axis, value),
        })
    }
}

#[derive(Debug, Default)]
pub struct GamepadCacheManager {
    pub gamepads: HashMap<u32, GamepadStateCache>,
}

impl GamepadCacheManager {
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
        }
    }

    pub fn add_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.insert(gamepad_id, GamepadStateCache::new());
    }

    pub fn remove_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.remove(&gamepad_id);
    }

    pub fn get_mut(&mut self, gamepad_id: u32) -> Option<&mut GamepadStateCache> {
        self.gamepads.get_mut(&gamepad_id)
    }
}

#[derive(Debug, Default)]
pub struct GamepadState {
    pub cache: GamepadCacheManager,
}

impl GamepadState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{GamepadEvent, GamepadStateCache};
    use crate::ElementState;

    #[test]
    fn update_axis_and_get_returns_current_adjusted_value() {
        let mut cache = GamepadStateCache::new();
        let value = cache.update_axis_and_get(0, 0.5);
        assert_eq!(value, cache.get_axis_value(0));
    }

    #[test]
    fn update_axis_and_get_applies_dead_zone() {
        let mut cache = GamepadStateCache::new();
        let value = cache.update_axis_and_get(0, 0.01);
        assert_eq!(value, 0.0);
    }

    #[test]
    fn update_button_event_emits_pressed_state() {
        let mut cache = GamepadStateCache::new();
        let event = cache.update_button_event(7, 2, 1.0);
        assert!(matches!(
            event,
            Some(GamepadEvent::OnButton {
                gamepad_id: 7,
                button: 2,
                state: ElementState::Pressed,
                value,
            }) if value == 1.0
        ));
    }

    #[test]
    fn update_axis_event_skips_small_changes() {
        let mut cache = GamepadStateCache::new();
        assert!(cache.update_axis_event(3, 1, 0.0).is_none());
    }
}
