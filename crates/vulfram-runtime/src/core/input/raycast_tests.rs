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
