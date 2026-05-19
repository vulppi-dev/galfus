#[cfg(not(target_arch = "wasm32"))]
use glam::Vec2;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use crate::ModifiersState;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyboardStateCache {
    pub modifiers: ModifiersState,
}

#[cfg(not(target_arch = "wasm32"))]
impl KeyboardStateCache {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct PointerStateCache {
    pub position: Vec2,
}

#[cfg(not(target_arch = "wasm32"))]
impl PointerStateCache {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
        }
    }

    pub fn position_changed(&self, new_pos: Vec2) -> bool {
        (self.position[0] - new_pos[0]).abs() > f32::EPSILON
            || (self.position[1] - new_pos[1]).abs() > f32::EPSILON
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default)]
pub struct InputCacheManager {
    pub keyboard: KeyboardStateCache,
    pub pointers: HashMap<u32, PointerStateCache>,
}

#[cfg(not(target_arch = "wasm32"))]
impl InputCacheManager {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardStateCache::new(),
            pointers: HashMap::new(),
        }
    }

    pub fn get_or_create_pointer(&mut self, window_id: u32) -> &mut PointerStateCache {
        self.pointers
            .entry(window_id)
            .or_insert_with(PointerStateCache::new)
    }

    pub fn remove_pointer(&mut self, window_id: u32) {
        self.pointers.remove(&window_id);
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default)]
pub struct InputCacheManager;

#[cfg(target_arch = "wasm32")]
impl InputCacheManager {
    pub fn new() -> Self {
        Self
    }

    pub fn remove_pointer(&mut self, _window_id: u32) {}
}

#[cfg(not(target_arch = "wasm32"))]
pub struct InputState {
    pub modifiers: ModifiersState,
    pub cache: InputCacheManager,
}

#[cfg(not(target_arch = "wasm32"))]
impl InputState {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::default(),
            cache: InputCacheManager::new(),
        }
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
