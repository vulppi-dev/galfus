use glam::{Vec2, Vec3, Vec4};

use crate::core::realm::RealmId;
use crate::core::resources::{PBR_INVALID_SLOT, STANDARD_INVALID_SLOT};
use crate::core::state::EngineState;
use crate::core::target::TargetId;

#[derive(Debug, Clone, Copy)]
pub(super) struct UiPlaneHit {
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

pub(super) fn resolve_ui_plane_hit(
    engine_state: &EngineState,
    window_id: u32,
    target_realm: RealmId,
    pointer_position: Vec2,
    pointer_surface_size: glam::UVec2,
) -> Option<UiPlaneHit> {
    let realm_entry = engine_state
        .universal_state
        .realms
        .entries
        .get(&target_realm)?;
    if realm_entry.value.kind != crate::core::realm::RealmKind::ThreeD {
        return None;
    }

    let window_state = engine_state.window.states.get(&window_id)?;
    let camera = pick_camera_for_pointer(window_state, pointer_position, pointer_surface_size)?;
    let ray = screen_ray_from_pointer(pointer_position, pointer_surface_size, camera)?;
    let mut ui_source_cache: std::collections::HashMap<u32, Option<UiTextureSource>> =
        std::collections::HashMap::new();

    let mut best_hit: Option<(f32, UiPlaneHit)> = None;
    let vertex = window_state.render_state.vertex.as_ref()?;

    for model in window_state.render_state.scene.models.values() {
        if (model.layer_mask & camera.layer_mask) == 0 {
            continue;
        }
        let Some(material_id) = model.material_id else {
            continue;
        };
        let ui_source = if let Some(cached) = ui_source_cache.get(&material_id) {
            *cached
        } else {
            let resolved = resolve_ui_source_for_model(engine_state, window_id, Some(material_id));
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
        let hit = UiPlaneHit {
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
    window_state: &crate::core::window::WindowState,
    pointer_position: Vec2,
    surface_size: glam::UVec2,
) -> Option<&crate::core::resources::CameraRecord> {
    let mut picked: Option<(&crate::core::resources::CameraRecord, i32, u32)> = None;
    for (camera_id, camera) in &window_state.render_state.scene.cameras {
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
    camera: &crate::core::resources::CameraRecord,
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

    let inv_vp = camera.data.view_projection.inverse();
    let near_h = inv_vp * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
    let far_h = inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    if near_h.w.abs() < 1e-6 || far_h.w.abs() < 1e-6 {
        return None;
    }

    let near = near_h.truncate() / near_h.w;
    let far = far_h.truncate() / far_h.w;
    let mut direction = (far - near).normalize_or_zero();
    if direction.length_squared() <= 1e-8 {
        return None;
    }

    let camera_forward = camera.data.direction.truncate().normalize_or_zero();
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
    camera: &crate::core::resources::CameraRecord,
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
    window_id: u32,
    material_id: Option<u32>,
) -> Option<UiTextureSource> {
    let material_id = material_id?;
    let window_state = engine_state.window.states.get(&window_id)?;
    let render_state = &window_state.render_state;

    if let Some(standard) = render_state.scene.materials_standard.get(&material_id) {
        for (slot_index, &texture_id) in standard.texture_ids.iter().enumerate() {
            if texture_id == STANDARD_INVALID_SLOT {
                continue;
            }
            if let Some(binding) = render_state.target_texture_binds.get(&texture_id)
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

    if let Some(pbr) = render_state.scene.materials_pbr.get(&material_id) {
        for (slot_index, &texture_id) in pbr.texture_ids.iter().enumerate() {
            if texture_id == PBR_INVALID_SLOT {
                continue;
            }
            if let Some(binding) = render_state.target_texture_binds.get(&texture_id)
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
        ((hit_local.y - aabb.min.y) / height).clamp(0.0, 1.0),
    );
    Some((world_distance, uv))
}
