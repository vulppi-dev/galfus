pub(crate) fn mark_global_materials_dirty(
    resources: &mut crate::core::render::Realm3dState,
    texture_id: u32,
) {
    for record in resources.materials.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
            record.mark_dirty();
        }
    }
}
