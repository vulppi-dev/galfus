use crate::meta::{
    CameraProjectionPlan, EntityRecordUpdatePlan, ForwardAtlasEntryMeta, LightRecordMeta,
    MaterialRecordMeta, MaterialRecordUpdatePlan, ModelRecordMeta, SyncPlan,
    TargetTextureBindingMeta, TextureRecordMeta,
};

pub fn collect_record_meta<T, M, F>(
    records: &std::collections::HashMap<u32, T>,
    mut to_meta: F,
) -> Vec<M>
where
    F: FnMut(u32, &T) -> M,
{
    records
        .iter()
        .map(|(id, record)| to_meta(*id, record))
        .collect()
}

pub fn hash_map_by_meta<T, M, F, H>(
    records: &std::collections::HashMap<u32, T>,
    to_meta: F,
    hash_meta: H,
) -> u64
where
    F: FnMut(u32, &T) -> M,
    H: Fn(&[M]) -> u64,
{
    let meta = collect_record_meta(records, to_meta);
    hash_meta(&meta)
}

pub fn retain_records_by_ids<T>(
    records: &mut std::collections::HashMap<u32, T>,
    live_ids: &std::collections::HashSet<u32>,
) {
    records.retain(|record_id, _| live_ids.contains(record_id));
}

pub fn rebuild_record_map<T: Clone, F>(
    current: &mut std::collections::HashMap<u32, T>,
    next: &std::collections::HashMap<u32, T>,
    mut update_existing: F,
) where
    F: FnMut(&mut T, &T),
{
    let mut previous = std::mem::take(current);
    current.clear();
    for (record_id, next_record) in next {
        if let Some(mut current_record) = previous.remove(record_id) {
            update_existing(&mut current_record, next_record);
            current.insert(*record_id, current_record);
        } else {
            current.insert(*record_id, next_record.clone());
        }
    }
}

pub fn sync_map_by_meta<T: Clone, M, F, P>(
    current: &mut std::collections::HashMap<u32, T>,
    next: &std::collections::HashMap<u32, T>,
    to_meta: F,
    plan_sync: P,
) where
    F: FnMut(u32, &T) -> M,
    P: Fn(&[M], &[M]) -> SyncPlan,
{
    let mut to_meta = to_meta;
    let current_meta = collect_record_meta(current, &mut to_meta);
    let next_meta = collect_record_meta(next, &mut to_meta);
    let plan = plan_sync(&current_meta, &next_meta);
    apply_sync_plan_map(current, next, &plan);
}

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

pub fn plan_model_record_update(
    current: &ModelRecordMeta,
    next: &ModelRecordMeta,
) -> EntityRecordUpdatePlan {
    EntityRecordUpdatePlan {
        mark_dirty: current != next,
    }
}

pub fn plan_light_record_update(
    current: &LightRecordMeta,
    next: &LightRecordMeta,
) -> EntityRecordUpdatePlan {
    EntityRecordUpdatePlan {
        mark_dirty: current != next,
    }
}

pub fn plan_material_record_update(
    current: &MaterialRecordMeta,
    next: &MaterialRecordMeta,
) -> MaterialRecordUpdatePlan {
    let mark_dirty = current != next;
    MaterialRecordUpdatePlan {
        mark_dirty,
        reset_bind_group: mark_dirty,
    }
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
