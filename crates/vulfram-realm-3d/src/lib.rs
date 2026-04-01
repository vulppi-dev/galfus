mod meta;
mod passes;
mod sync;

pub use meta::{
    CameraProjectionPlan, EntityRecordUpdatePlan, ForwardAtlasEntryMeta, LightRecordMeta,
    MaterialRecordMeta, MaterialRecordUpdatePlan, ModelRecordMeta, SyncPlan,
    TargetTextureBindingMeta, TextureRecordMeta,
};
pub use passes::{SUPPORTED_RENDER_PASSES, graph_is_compatible, supports_render_pass};
pub use sync::{
    apply_sync_plan_map, collect_record_meta, hash_forward_atlas_entries, hash_map_by_meta,
    hash_target_texture_binds, hash_texture_records, plan_camera_projection_update,
    plan_forward_atlas_sync, plan_geometry_registry_sync, plan_light_record_update,
    plan_material_record_update, plan_model_record_update, plan_target_texture_bind_sync,
    plan_texture_record_sync, rebuild_record_map, retain_records_by_ids, sync_map_by_meta,
};

#[cfg(test)]
mod tests;
