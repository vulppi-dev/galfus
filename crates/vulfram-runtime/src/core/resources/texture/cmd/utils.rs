pub(crate) fn mark_global_materials_dirty(
    resources: &mut crate::core::realm::Realm3dState,
    texture_id: u32,
) {
    for record in resources.materials_standard.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
            record.mark_dirty();
        }
    }
    for record in resources.materials_pbr.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
            record.mark_dirty();
        }
    }
}
