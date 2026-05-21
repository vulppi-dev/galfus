use bytemuck;
use glam::{Vec2, Vec3};

use crate::core::resources::geometry::primitives::PillOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

use super::compute_tangents;

pub fn generate_pill(options: &PillOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let radius = options.radius.max(0.0001);
    let height = options.height.max(0.0);
    let sectors = options.sectors.max(3);
    let stacks = options.stacks.max(1);
    let half_cylinder = height * 0.5;

    let mut rings: Vec<(f32, f32)> = Vec::new();

    rings.push((half_cylinder + radius, 0.0));
    for i in 1..=stacks {
        let theta = std::f32::consts::FRAC_PI_2 * (i as f32 / stacks as f32);
        let y = half_cylinder + radius * theta.cos();
        let r = radius * theta.sin();
        rings.push((y, r));
    }

    if half_cylinder > 0.0 {
        rings.push((-half_cylinder, radius));
    }

    for i in 1..=stacks {
        let theta = std::f32::consts::FRAC_PI_2 * (i as f32 / stacks as f32);
        let y = -half_cylinder - radius * theta.sin();
        let r = radius * theta.cos();
        rings.push((y, r));
    }

    let ring_count = rings.len().max(2);
    let mut positions = Vec::with_capacity(ring_count * (sectors as usize + 1));
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity((ring_count - 1) * sectors as usize * 6);

    for (ring_index, (y, ring_radius)) in rings.iter().copied().enumerate() {
        let v = if ring_count > 1 {
            ring_index as f32 / (ring_count as f32 - 1.0)
        } else {
            0.0
        };
        for sector in 0..=sectors {
            let u = sector as f32 / sectors as f32;
            let angle = u * std::f32::consts::TAU;
            let (sin_a, cos_a) = angle.sin_cos();

            let x = ring_radius * cos_a;
            let z = ring_radius * sin_a;
            let pos = Vec3::new(x, y, z);
            positions.push(pos);

            let normal = if y > half_cylinder {
                Vec3::new(x, y - half_cylinder, z).normalize_or_zero()
            } else if y < -half_cylinder {
                Vec3::new(x, y + half_cylinder, z).normalize_or_zero()
            } else {
                Vec3::new(cos_a, 0.0, sin_a).normalize_or_zero()
            };
            normals.push(normal);
            uvs.push(Vec2::new(u, v));
        }
    }

    let stride = sectors + 1;
    for ring in 0..(ring_count as u32 - 1) {
        let row = ring * stride;
        let next_row = (ring + 1) * stride;
        for sector in 0..sectors {
            let i0 = row + sector;
            let i1 = i0 + 1;
            let i2 = next_row + sector;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i1);
            indices.push(i2);

            indices.push(i1);
            indices.push(i3);
            indices.push(i2);
        }
    }

    let tangents = compute_tangents(&positions, &normals, &uvs, &indices);

    vec![
        (
            GeometryPrimitiveType::Position,
            bytemuck::cast_slice(&positions).to_vec(),
        ),
        (
            GeometryPrimitiveType::Normal,
            bytemuck::cast_slice(&normals).to_vec(),
        ),
        (
            GeometryPrimitiveType::UV,
            bytemuck::cast_slice(&uvs).to_vec(),
        ),
        (
            GeometryPrimitiveType::Tangent,
            bytemuck::cast_slice(&tangents).to_vec(),
        ),
        (
            GeometryPrimitiveType::Index,
            bytemuck::cast_slice(&indices).to_vec(),
        ),
    ]
}
