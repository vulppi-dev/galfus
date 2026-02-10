use glam::{Vec2, Vec3, Vec4};

pub fn build_skinned_plane(
    width_segments: u32,
    depth_segments: u32,
    size: f32,
    bone_count: u32,
) -> (
    Vec<Vec3>,
    Vec<Vec3>,
    Vec<Vec2>,
    Vec<[u16; 4]>,
    Vec<Vec4>,
    Vec<u32>,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    let half_size = size * 0.5;
    let w = width_segments.max(1) as usize;
    let d = depth_segments.max(1) as usize;

    for z in 0..=d {
        let v = z as f32 / d as f32;
        let z_pos = v * size - half_size;

        for x in 0..=w {
            let u = x as f32 / w as f32;
            let x_pos = u * size - half_size;

            positions.push(Vec3::new(x_pos, 0.0, z_pos));
            normals.push(Vec3::Y);
            uvs.push(Vec2::new(u, v));

            let bone_span = (bone_count.max(1) - 1) as f32;
            let bone_float = u * bone_span;
            let bone_index = bone_float.floor() as u32;
            let next_index = (bone_index + 1).min(bone_count.saturating_sub(1));
            let weight = bone_float - bone_index as f32;

            joints.push([bone_index as u16, next_index as u16, 0, 0]);
            weights.push(Vec4::new(1.0 - weight, weight, 0.0, 0.0));
        }
    }

    let row = w + 1;
    for z in 0..d {
        for x in 0..w {
            let i0 = (z * row + x) as u32;
            let i1 = i0 + 1;
            let i2 = ((z + 1) * row + x) as u32;
            let i3 = i2 + 1;

            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    (positions, normals, uvs, joints, weights, indices)
}
