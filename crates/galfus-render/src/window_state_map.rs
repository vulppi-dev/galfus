use std::collections::HashMap;

pub struct WindowStateMap<T> {
    pub states: HashMap<u32, T>,
}

impl<T> Default for WindowStateMap<T> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
        }
    }
}

impl<T> WindowStateMap<T> {
    pub fn insert(&mut self, window_id: u32, state: T) {
        self.states.insert(window_id, state);
    }

    pub fn remove(&mut self, window_id: u32) -> Option<T> {
        self.states.remove(&window_id)
    }

    pub fn get(&self, window_id: &u32) -> Option<&T> {
        self.states.get(window_id)
    }

    pub fn get_mut(&mut self, window_id: &u32) -> Option<&mut T> {
        self.states.get_mut(window_id)
    }
}
