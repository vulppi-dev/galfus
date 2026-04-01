use crate::meta::{
    CameraProjectionPlan, ForwardAtlasEntryMeta, LightRecordMeta, MaterialRecordMeta,
    ModelRecordMeta, SyncPlan, TargetTextureBindingMeta, TextureRecordMeta,
};

pub fn hash_texture_records(records: &[TextureRecordMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    records.len().hash(&mut hasher);
    for record in records {
        record.id.hash(&mut hasher);
        record.label.hash(&mut hasher);
        record.width.hash(&mut hasher);
        record.height.hash(&mut hasher);
        record.depth_or_array_layers.hash(&mut hasher);
        record.format.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn hash_forward_atlas_entries(entries: &[ForwardAtlasEntryMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entries.len().hash(&mut hasher);
    for entry in entries {
        entry.id.hash(&mut hasher);
        entry.label.hash(&mut hasher);
        entry.layer.hash(&mut hasher);
        bytemuck::bytes_of(&entry.uv_scale_bias).hash(&mut hasher);
    }
    hasher.finish()
}

pub fn hash_target_texture_binds(binds: &[TargetTextureBindingMeta]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    binds.len().hash(&mut hasher);
    for bind in binds {
        bind.texture_id.hash(&mut hasher);
        bind.target_id.hash(&mut hasher);
        bind.label.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn plan_camera_projection_update(
    previous_kind_flags: [u32; 2],
    previous_near_far: [f32; 2],
    previous_ortho_scale: f32,
    next_kind_flags: [u32; 2],
    next_near_far: [f32; 2],
    next_ortho_scale: f32,
    previous_projection_size: [u32; 2],
) -> CameraProjectionPlan {
    let projection_params_changed = previous_kind_flags != next_kind_flags
        || previous_near_far != next_near_far
        || (previous_ortho_scale - next_ortho_scale).abs() > f32::EPSILON;
    let has_previous_projection =
        previous_projection_size[0] > 0 && previous_projection_size[1] > 0;

    CameraProjectionPlan {
        preserve_runtime_projection: has_previous_projection && !projection_params_changed,
        reset_projection_size: projection_params_changed,
    }
}

pub fn model_record_changed(current: &ModelRecordMeta, next: &ModelRecordMeta) -> bool {
    current != next
}

pub fn light_record_changed(current: &LightRecordMeta, next: &LightRecordMeta) -> bool {
    current != next
}

pub fn material_record_changed(current: &MaterialRecordMeta, next: &MaterialRecordMeta) -> bool {
    current != next
}

pub fn plan_texture_record_sync(
    current: &[TextureRecordMeta],
    next: &[TextureRecordMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|record| (record.id, record)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|record| (record.id, record)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|record| match current_by_id.get(&record.id) {
            Some(current) => *current != *record,
            None => true,
        })
        .map(|record| record.id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_forward_atlas_sync(
    current: &[ForwardAtlasEntryMeta],
    next: &[ForwardAtlasEntryMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|entry| (entry.id, entry)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|entry| (entry.id, entry)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|entry| match current_by_id.get(&entry.id) {
            Some(current) => *current != *entry,
            None => true,
        })
        .map(|entry| entry.id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_target_texture_bind_sync(
    current: &[TargetTextureBindingMeta],
    next: &[TargetTextureBindingMeta],
) -> SyncPlan {
    let current_by_id: std::collections::HashMap<_, _> =
        current.iter().map(|bind| (bind.texture_id, bind)).collect();
    let next_by_id: std::collections::HashMap<_, _> =
        next.iter().map(|bind| (bind.texture_id, bind)).collect();

    let mut stale_ids: Vec<u32> = current_by_id
        .keys()
        .filter(|id| !next_by_id.contains_key(id))
        .copied()
        .collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next
        .iter()
        .filter(|bind| match current_by_id.get(&bind.texture_id) {
            Some(current) => *current != *bind,
            None => true,
        })
        .map(|bind| bind.texture_id)
        .collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn plan_geometry_registry_sync(current_ids: &[u32], next_ids: &[u32]) -> SyncPlan {
    let current_ids: std::collections::HashSet<u32> = current_ids.iter().copied().collect();
    let next_ids: std::collections::HashSet<u32> = next_ids.iter().copied().collect();

    let mut stale_ids: Vec<u32> = current_ids.difference(&next_ids).copied().collect();
    stale_ids.sort_unstable();

    let mut replace_ids: Vec<u32> = next_ids.difference(&current_ids).copied().collect();
    replace_ids.sort_unstable();

    SyncPlan {
        stale_ids,
        replace_ids,
    }
}

pub fn apply_sync_plan_map<T: Clone>(
    current: &mut std::collections::HashMap<u32, T>,
    next: &std::collections::HashMap<u32, T>,
    plan: &SyncPlan,
) {
    for stale_id in &plan.stale_ids {
        current.remove(stale_id);
    }
    for replace_id in &plan.replace_ids {
        if let Some(record) = next.get(replace_id) {
            current.insert(*replace_id, record.clone());
        }
    }
}
