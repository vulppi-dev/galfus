use crate::core::render::RenderState;
use std::collections::HashMap;

#[derive(Default)]
pub struct RenderManager {
    pub states: HashMap<u32, RenderState>,
}

impl RenderManager {
    pub fn insert(&mut self, window_id: u32, render_state: RenderState) {
        self.states.insert(window_id, render_state);
    }

    pub fn remove(&mut self, window_id: u32) -> Option<RenderState> {
        self.states.remove(&window_id)
    }

    pub fn get_mut(&mut self, window_id: &u32) -> Option<&mut RenderState> {
        self.states.get_mut(window_id)
    }

    pub fn get(&self, window_id: &u32) -> Option<&RenderState> {
        self.states.get(window_id)
    }
}
