use super::resolve_target_size;
use crate::core::realm::RealmId;
use crate::core::target::TargetId;
use crate::core::test_support::{
    alloc_offscreen_surface, insert_texture_target, link_target_surface, test_engine,
};
use glam::UVec2;

#[test]
fn resolve_target_size_prefers_target_surface_over_declared_size() {
    let mut engine = test_engine();
    let target_id = TargetId(700);
    insert_texture_target(&mut engine, target_id, UVec2::new(300, 200));

    let surface_id = alloc_offscreen_surface(&mut engine, UVec2::new(1280, 720));
    link_target_surface(&mut engine, RealmId(1), target_id, surface_id);

    let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
    assert_eq!(size, Some(UVec2::new(1280, 720)));
}

#[test]
fn resolve_target_size_falls_back_to_declared_without_runtime_surface() {
    let mut engine = test_engine();
    let target_id = TargetId(701);
    insert_texture_target(&mut engine, target_id, UVec2::new(640, 360));

    let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
    assert_eq!(size, Some(UVec2::new(640, 360)));
}
