use std::collections::HashSet;

use crate::core::state::EngineState;
use crate::core::target::TargetKind;

pub fn default_layer_mask() -> u32 {
    0xFFFFFFFF
}

pub fn default_true() -> bool {
    true
}

pub fn default_vec4_zero() -> glam::Vec4 {
    glam::Vec4::ZERO
}

pub fn mark_realm_windows_dirty(engine: &mut EngineState, realm_id: u32) {
    let mut dirty_windows: HashSet<u32> = HashSet::new();

    if let Some(realm) = engine
        .universal_state
        .realms
        .entries
        .get(&crate::core::realm::RealmId(realm_id))
        && let Some(window_id) = realm.value.host_window_id
    {
        dirty_windows.insert(window_id);
    }

    for ((layer_realm_id, target_id), _) in &engine.universal_state.target_layers.entries {
        if *layer_realm_id != realm_id {
            continue;
        }
        let Some(target) = engine.universal_state.targets.entries.get(target_id) else {
            continue;
        };
        if target.kind != TargetKind::Window {
            continue;
        }
        if let Some(window_id) = target.window_id {
            dirty_windows.insert(window_id);
        }
    }

    for window_id in dirty_windows {
        if let Some(window_state) = engine.window.states.get_mut(&window_id) {
            window_state.is_dirty = true;
        }
    }
}
