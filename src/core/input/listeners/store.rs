use std::collections::HashMap;

use crate::core::target::TargetId;

use super::model::{InputTargetListenerConfig, InputTargetListenerSnapshot};

#[derive(Debug, Default)]
pub struct InputTargetListenerStore {
    by_listener: HashMap<u64, InputTargetListenerConfig>,
    listeners_by_target: HashMap<TargetId, Vec<u64>>,
}

impl InputTargetListenerStore {
    pub fn upsert(&mut self, config: InputTargetListenerConfig) {
        if let Some(previous) = self.by_listener.insert(config.listener_id, config.clone()) {
            self.remove_from_target_index(previous.target_id, previous.listener_id);
        }
        self.listeners_by_target
            .entry(config.target_id)
            .or_default()
            .push(config.listener_id);
    }

    pub fn dispose(&mut self, listener_id: u64) -> bool {
        let Some(config) = self.by_listener.remove(&listener_id) else {
            return false;
        };
        self.remove_from_target_index(config.target_id, listener_id);
        true
    }

    pub fn dispose_target(&mut self, target_id: TargetId) -> usize {
        let Some(listener_ids) = self.listeners_by_target.remove(&target_id) else {
            return 0;
        };
        let mut removed = 0;
        for listener_id in listener_ids {
            if self.by_listener.remove(&listener_id).is_some() {
                removed += 1;
            }
        }
        removed
    }

    pub fn dispose_targets<I: IntoIterator<Item = TargetId>>(&mut self, target_ids: I) -> usize {
        target_ids
            .into_iter()
            .map(|target_id| self.dispose_target(target_id))
            .sum()
    }

    pub fn list(&self, target_id: Option<TargetId>) -> Vec<InputTargetListenerSnapshot> {
        let mut listeners = match target_id {
            Some(target_id) => self
                .listeners_by_target
                .get(&target_id)
                .into_iter()
                .flat_map(|listener_ids| listener_ids.iter())
                .filter_map(|listener_id| self.by_listener.get(listener_id))
                .map(to_snapshot)
                .collect::<Vec<_>>(),
            None => self
                .by_listener
                .values()
                .map(to_snapshot)
                .collect::<Vec<_>>(),
        };
        listeners.sort_by_key(|listener| listener.listener_id);
        listeners
    }

    pub fn listeners_for_target(&self, target_id: TargetId) -> Vec<InputTargetListenerConfig> {
        self.listeners_by_target
            .get(&target_id)
            .into_iter()
            .flat_map(|listener_ids| listener_ids.iter())
            .filter_map(|listener_id| self.by_listener.get(listener_id).cloned())
            .collect()
    }

    fn remove_from_target_index(&mut self, target_id: TargetId, listener_id: u64) {
        if let Some(listener_ids) = self.listeners_by_target.get_mut(&target_id) {
            listener_ids.retain(|id| *id != listener_id);
            if listener_ids.is_empty() {
                self.listeners_by_target.remove(&target_id);
            }
        }
    }
}

fn to_snapshot(config: &InputTargetListenerConfig) -> InputTargetListenerSnapshot {
    InputTargetListenerSnapshot {
        listener_id: config.listener_id,
        target_id: config.target_id.0,
        enabled: config.enabled,
        events: config.events.clone(),
        sample_percent: config.sample_percent,
    }
}
