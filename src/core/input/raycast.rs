use glam::{Vec2, Vec3, Vec4};

use crate::core::realm::RealmId;
use crate::core::resources::{PBR_INVALID_SLOT, STANDARD_INVALID_SLOT};
use crate::core::state::EngineState;
use crate::core::target::TargetId;

#[derive(Debug, Clone, Copy)]
pub(super) struct RealmPlaneHit {
    pub source_realm_id: RealmId,
    pub target_id: TargetId,
    pub uv: Vec2,
}

#[derive(Debug, Clone, Copy)]
struct UiTextureSource {
    source_realm_id: RealmId,
    target_id: TargetId,
    uv_scale_bias: Vec4,
}

#[derive(Debug, Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

pub(super) fn resolve_realm_plane_hit(
    engine_state: &EngineState,
    window_id: u32,
    target_realm: RealmId,
    preferred_camera_id: Option<u32>,
    pointer_position: Vec2,
    pointer_surface_size: glam::UVec2,
) -> Option<RealmPlaneHit> {
    let realm_entry = engine_state
        .universal_state
        .realms
        .entries
        .get(&target_realm)?;
    if realm_entry.value.kind != crate::core::realm::RealmKind::ThreeD {
        return None;
    }

    let render_state = engine_state.render.get(&window_id)?;
    let entities = engine_state
        .universal_state
        .realm_entities
        .get(&target_realm)?;
    let camera = pick_camera_for_pointer(
        &entities.cameras,
        preferred_camera_id,
        pointer_position,
        pointer_surface_size,
    )?;
    let ray = screen_ray_from_pointer(pointer_position, pointer_surface_size, camera)?;
    let mut ui_source_cache: std::collections::HashMap<u32, Option<UiTextureSource>> =
        std::collections::HashMap::new();

    let mut best_hit: Option<(f32, RealmPlaneHit)> = None;
    let vertex = render_state.vertex.as_ref()?;

    for model in entities.models.values() {
        if (model.layer_mask & camera.layer_mask) == 0 {
            continue;
        }
        let Some(material_id) = model.material_id else {
            continue;
        };
        let ui_source = if let Some(cached) = ui_source_cache.get(&material_id) {
            *cached
        } else {
            let resolved = resolve_ui_source_for_model(engine_state, Some(material_id));
            ui_source_cache.insert(material_id, resolved);
            resolved
        };
        let Some(ui_source) = ui_source else {
            continue;
        };
        let Some(aabb) = vertex.aabb(model.geometry_id) else {
            continue;
        };
        let depth = (aabb.max.z - aabb.min.z).abs();
        if depth > 0.0001 {
            continue;
        }
        let Some((distance, uv)) = intersect_plane_model(&ray, model.data.transform, aabb) else {
            continue;
        };

        let mapped_uv = apply_uv_scale_bias(uv, ui_source.uv_scale_bias);
        let hit = RealmPlaneHit {
            source_realm_id: ui_source.source_realm_id,
            target_id: ui_source.target_id,
            uv: mapped_uv,
        };
        match best_hit {
            Some((best_distance, _)) if distance >= best_distance => {}
            _ => best_hit = Some((distance, hit)),
        }
    }

    best_hit.map(|(_, hit)| hit)
}

fn pick_camera_for_pointer(
    cameras: &std::collections::HashMap<u32, crate::core::resources::CameraNode>,
    preferred_camera_id: Option<u32>,
    pointer_position: Vec2,
    surface_size: glam::UVec2,
) -> Option<&crate::core::resources::CameraNode> {
    if let Some(camera_id) = preferred_camera_id {
        if let Some(camera) = cameras.get(&camera_id) {
            return Some(camera);
        }
    }
    let mut picked: Option<(&crate::core::resources::CameraNode, i32, u32)> = None;
    for (camera_id, camera) in cameras {
        let (viewport_x, viewport_y, viewport_w, viewport_h) =
            resolve_camera_viewport_in_surface(surface_size, camera);
        if pointer_position.x >= viewport_x
            && pointer_position.x <= viewport_x + viewport_w
            && pointer_position.y >= viewport_y
            && pointer_position.y <= viewport_y + viewport_h
        {
            match picked {
                None => picked = Some((camera, camera.order, *camera_id)),
                Some((_, best_order, best_id)) => {
                    if camera.order > best_order
                        || (camera.order == best_order && *camera_id > best_id)
                    {
                        picked = Some((camera, camera.order, *camera_id));
                    }
                }
            }
        }
    }
    picked.map(|(camera, _, _)| camera)
}

fn screen_ray_from_pointer(
    pointer_position: Vec2,
    surface_size: glam::UVec2,
    camera: &crate::core::resources::CameraNode,
) -> Option<Ray> {
    let (viewport_x, viewport_y, viewport_w, viewport_h) =
        resolve_camera_viewport_in_surface(surface_size, camera);
    if pointer_position.x < viewport_x
        || pointer_position.x > viewport_x + viewport_w
        || pointer_position.y < viewport_y
        || pointer_position.y > viewport_y + viewport_h
    {
        return None;
    }

    let local_x = pointer_position.x - viewport_x;
    let local_y = pointer_position.y - viewport_y;
    let width = viewport_w.max(1.0);
    let height = viewport_h.max(1.0);
    let ndc_x = (local_x / width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (local_y / height) * 2.0;

    // Recompute projection using the effective viewport size so picking matches
    // the actual camera target used for rendering (e.g. widget viewport targets).
    let mut camera_data = camera.data;
    camera_data.update(
        None,
        None,
        None,
        None,
        (
            width.round().max(1.0) as u32,
            height.round().max(1.0) as u32,
        ),
        camera.ortho_scale,
    );

    let inv_vp = camera_data.view_projection.inverse();
    // Reverse-Z (WGPU): near=1, far=0 in NDC depth.
    let near_h = inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    let far_h = inv_vp * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
    if near_h.w.abs() < 1e-6 || far_h.w.abs() < 1e-6 {
        return None;
    }

    let near = near_h.truncate() / near_h.w;
    let far = far_h.truncate() / far_h.w;
    let mut direction = (far - near).normalize_or_zero();
    if direction.length_squared() <= 1e-8 {
        return None;
    }

    let camera_forward = camera_data.direction.truncate().normalize_or_zero();
    if camera_forward.length_squared() > 1e-8 && direction.dot(camera_forward) < 0.0 {
        direction = -direction;
    }

    Some(Ray {
        origin: camera.data.position.truncate(),
        direction,
    })
}

fn resolve_camera_viewport_in_surface(
    surface_size: glam::UVec2,
    camera: &crate::core::resources::CameraNode,
) -> (f32, f32, f32, f32) {
    if let Some(view_position) = camera.view_position.as_ref() {
        let (x, y) = view_position.resolve_position(surface_size.x, surface_size.y);
        let (w, h) = view_position.resolve_size(surface_size.x, surface_size.y);
        return (x as f32, y as f32, w.max(1) as f32, h.max(1) as f32);
    }
    (
        0.0,
        0.0,
        surface_size.x.max(1) as f32,
        surface_size.y.max(1) as f32,
    )
}

fn resolve_ui_source_for_model(
    engine_state: &EngineState,
    material_id: Option<u32>,
) -> Option<UiTextureSource> {
    let material_id = material_id?;
    let resources = &engine_state.universal_state.universal_resources;

    if let Some(standard) = resources.materials_standard.get(&material_id) {
        for (slot_index, &texture_id) in standard.texture_ids.iter().enumerate() {
            if texture_id == STANDARD_INVALID_SLOT {
                continue;
            }
            if let Some(binding) = resources.target_texture_binds.get(&texture_id)
                && let Some(realm_id) = resolve_target_ui_realm(engine_state, binding.target_id)
            {
                return Some(UiTextureSource {
                    source_realm_id: realm_id,
                    target_id: binding.target_id,
                    uv_scale_bias: standard.data.atlas_scale_bias[slot_index],
                });
            }
        }
    }

    if let Some(pbr) = resources.materials_pbr.get(&material_id) {
        for (slot_index, &texture_id) in pbr.texture_ids.iter().enumerate() {
            if texture_id == PBR_INVALID_SLOT {
                continue;
            }
            if let Some(binding) = resources.target_texture_binds.get(&texture_id)
                && let Some(realm_id) = resolve_target_ui_realm(engine_state, binding.target_id)
            {
                return Some(UiTextureSource {
                    source_realm_id: realm_id,
                    target_id: binding.target_id,
                    uv_scale_bias: pbr.data.atlas_scale_bias[slot_index],
                });
            }
        }
    }

    None
}

fn apply_uv_scale_bias(uv: Vec2, scale_bias: Vec4) -> Vec2 {
    let mapped = uv * Vec2::new(scale_bias.x, scale_bias.y) + Vec2::new(scale_bias.z, scale_bias.w);
    Vec2::new(mapped.x.clamp(0.0, 1.0), mapped.y.clamp(0.0, 1.0))
}

fn resolve_target_ui_realm(engine_state: &EngineState, target_id: TargetId) -> Option<RealmId> {
    if let Some(realm_id) = engine_state
        .universal_state
        .target_ui_realm_index
        .get(&target_id)
        .copied()
    {
        return Some(realm_id);
    }

    for layer in engine_state.universal_state.target_layers.entries.values() {
        if layer.target_id != target_id {
            continue;
        }
        let realm_id = RealmId(layer.realm_id);
        let Some(realm) = engine_state.universal_state.realms.entries.get(&realm_id) else {
            continue;
        };
        if realm.value.kind == crate::core::realm::RealmKind::TwoD {
            return Some(realm_id);
        }
    }
    None
}

fn intersect_plane_model(
    ray: &Ray,
    model_transform: glam::Mat4,
    aabb: crate::core::resources::geometry::Aabb,
) -> Option<(f32, Vec2)> {
    let det = model_transform.determinant();
    if det.abs() < 1e-8 {
        return None;
    }

    let inv = model_transform.inverse();
    let origin_local = inv.transform_point3(ray.origin);
    let direction_local = inv.transform_vector3(ray.direction);
    if direction_local.z.abs() < 1e-6 {
        return None;
    }
    // RealmPlane is double-sided in rasterization, but pointer interaction is
    // only valid for the front face (local +Z normal).
    if direction_local.z >= -1e-6 {
        return None;
    }

    let local_t = -origin_local.z / direction_local.z;
    if local_t <= 0.0 {
        return None;
    }

    let hit_local = origin_local + direction_local * local_t;
    if hit_local.x < aabb.min.x
        || hit_local.x > aabb.max.x
        || hit_local.y < aabb.min.y
        || hit_local.y > aabb.max.y
    {
        return None;
    }

    let world_hit = model_transform.transform_point3(hit_local);
    let world_distance = (world_hit - ray.origin).dot(ray.direction);
    if world_distance <= 0.0 {
        return None;
    }

    let width = (aabb.max.x - aabb.min.x).max(1e-6);
    let height = (aabb.max.y - aabb.min.y).max(1e-6);
    let uv = Vec2::new(
        ((hit_local.x - aabb.min.x) / width).clamp(0.0, 1.0),
        1.0 - ((hit_local.y - aabb.min.y) / height).clamp(0.0, 1.0),
    );
    Some((world_distance, uv))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_raycast_intersection_produces_uv() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 2.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
        };
        let aabb = crate::core::resources::geometry::Aabb {
            min: Vec3::new(-1.0, -1.0, 0.0),
            max: Vec3::new(1.0, 1.0, 0.0),
        };
        let hit = intersect_plane_model(&ray, glam::Mat4::IDENTITY, aabb);
        assert!(hit.is_some());
        let (_distance, uv) = hit.unwrap();
        assert!((uv.x - 0.5).abs() < 1e-4);
        assert!((uv.y - 0.5).abs() < 1e-4);
    }

    #[test]
    fn plane_raycast_applies_uv_scale_bias() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 2.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
        };
        let aabb = crate::core::resources::geometry::Aabb {
            min: Vec3::new(-1.0, -1.0, -0.5),
            max: Vec3::new(1.0, 1.0, 0.5),
        };
        let hit = intersect_plane_model(&ray, glam::Mat4::IDENTITY, aabb);
        assert!(hit.is_some());
        let mapped = apply_uv_scale_bias(hit.unwrap().1, Vec4::new(0.5, 0.5, 0.25, 0.25));
        assert!(mapped.x >= 0.25 && mapped.x <= 0.75);
        assert!(mapped.y >= 0.25 && mapped.y <= 0.75);
    }
}
