mod meta;
mod passes;
mod sync;

pub use meta::{
    CameraProjectionPlan, ForwardAtlasEntryMeta, LightRecordMeta, MaterialRecordMeta,
    ModelRecordMeta, SyncPlan, TargetTextureBindingMeta, TextureRecordMeta,
};
pub use passes::{SUPPORTED_RENDER_PASSES, graph_is_compatible, supports_render_pass};
pub use sync::{
    apply_sync_plan_map, hash_forward_atlas_entries, hash_target_texture_binds,
    hash_texture_records, light_record_changed, material_record_changed, model_record_changed,
    plan_camera_projection_update, plan_forward_atlas_sync, plan_geometry_registry_sync,
    plan_target_texture_bind_sync, plan_texture_record_sync,
};

#[cfg(test)]
mod tests;
