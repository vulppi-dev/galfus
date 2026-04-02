use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GizmoVertex {
    pub position: Vec3,
    pub _pad: f32,
    pub color: Vec4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGizmoDrawLineArgs {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
    #[serde(default)]
    pub thickness: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGizmoDrawAabbArgs {
    pub min: Vec3,
    pub max: Vec3,
    pub color: Vec4,
    #[serde(default)]
    pub thickness: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGizmoDrawPolylineArgs {
    pub points: Vec<Vec3>,
    pub color: Vec4,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub thickness: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultGizmoDraw {
    pub status: u32,
}

#[derive(Clone, Copy, Debug)]
struct GizmoSegment {
    start: Vec3,
    end: Vec3,
    color: Vec4,
    thickness_px: f32,
}

pub struct GizmoSystem {
    segments: Vec<GizmoSegment>,
    vertices: Vec<GizmoVertex>,
    buffer: Option<wgpu::Buffer>,
    capacity: usize,
}

impl GizmoSystem {
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(1024),
            vertices: Vec::with_capacity(1024),
            buffer: None,
            capacity: 0,
        }
    }

    pub fn add_line(&mut self, start: Vec3, end: Vec3, color: Vec4, thickness_px: f32) {
        self.segments.push(GizmoSegment {
            start,
            end,
            color,
            thickness_px: thickness_px.max(0.0),
        });
    }

    pub fn add_aabb(&mut self, min: Vec3, max: Vec3, color: Vec4, thickness_px: f32) {
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];

        self.add_line(corners[0], corners[1], color, thickness_px);
        self.add_line(corners[1], corners[2], color, thickness_px);
        self.add_line(corners[2], corners[3], color, thickness_px);
        self.add_line(corners[3], corners[0], color, thickness_px);

        self.add_line(corners[4], corners[5], color, thickness_px);
        self.add_line(corners[5], corners[6], color, thickness_px);
        self.add_line(corners[6], corners[7], color, thickness_px);
        self.add_line(corners[7], corners[4], color, thickness_px);

        self.add_line(corners[0], corners[4], color, thickness_px);
        self.add_line(corners[1], corners[5], color, thickness_px);
        self.add_line(corners[2], corners[6], color, thickness_px);
        self.add_line(corners[3], corners[7], color, thickness_px);
    }

    pub fn add_polyline(&mut self, points: &[Vec3], color: Vec4, closed: bool, thickness_px: f32) {
        if points.len() < 2 {
            return;
        }
        for segment in points.windows(2) {
            self.add_line(segment[0], segment[1], color, thickness_px);
        }
        if closed {
            let start = points[0];
            let end = points[points.len() - 1];
            if start != end {
                self.add_line(end, start, color, thickness_px);
            }
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.vertices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn prepare_for_camera(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &crate::core::resources::CameraRecord,
        viewport: glam::UVec2,
    ) {
        if self.segments.is_empty() {
            self.vertices.clear();
            return;
        }

        self.vertices.clear();

        let view_dir = camera.data.direction.truncate().normalize_or_zero();
        let camera_up = camera.data.up.truncate().normalize_or_zero();
        let camera_pos = camera.data.position.truncate();
        let kind = crate::core::resources::CameraKind::from_u32(camera.data.kind_flags.x)
            .unwrap_or(crate::core::resources::CameraKind::Perspective);
        let proj = camera.data.projection.to_cols_array_2d();
        let proj_yy = proj[1][1].abs().max(1e-6);
        let viewport_h = viewport.y.max(1) as f32;

        for index in 0..self.segments.len() {
            let seg = self.segments[index];
            let line_dir = (seg.end - seg.start).normalize_or_zero();
            if line_dir.length_squared() < 1e-8 {
                continue;
            }
            let mut perp = line_dir.cross(view_dir).normalize_or_zero();
            if perp.length_squared() < 1e-8 {
                perp = line_dir.cross(camera_up).normalize_or_zero();
            }
            if perp.length_squared() < 1e-8 {
                continue;
            }
            let half_thickness_px = seg.thickness_px.max(1.0) * 0.5;
            let Some(offset_start) = self.pixel_to_world_offset(
                kind,
                seg.start,
                camera_pos,
                view_dir,
                proj_yy,
                viewport_h,
                half_thickness_px,
            ) else {
                continue;
            };
            let Some(offset_end) = self.pixel_to_world_offset(
                kind,
                seg.end,
                camera_pos,
                view_dir,
                proj_yy,
                viewport_h,
                half_thickness_px,
            ) else {
                continue;
            };
            self.push_segment_quad(
                seg.start,
                seg.end,
                line_dir,
                perp,
                offset_start,
                offset_end,
                seg.color,
            );
        }

        if self.vertices.is_empty() {
            return;
        }

        if self.buffer.is_none() || self.capacity < self.vertices.len() {
            self.capacity = self.vertices.len().next_power_of_two();
            let size = self.capacity * std::mem::size_of::<GizmoVertex>();
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Gizmo Vertex Buffer"),
                size: size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
    }

    pub fn estimated_gpu_bytes(&self) -> u64 {
        self.buffer
            .as_ref()
            .map(|buffer| buffer.size())
            .unwrap_or(0)
    }

    fn pixel_to_world_offset(
        &self,
        kind: crate::core::resources::CameraKind,
        world_pos: Vec3,
        camera_pos: Vec3,
        camera_dir: Vec3,
        proj_yy: f32,
        viewport_h: f32,
        pixel_offset: f32,
    ) -> Option<f32> {
        match kind {
            crate::core::resources::CameraKind::Perspective => {
                let depth = (world_pos - camera_pos).dot(camera_dir);
                if depth <= 1e-4 {
                    return None;
                }
                let world_per_px = 2.0 * depth / (proj_yy * viewport_h);
                Some(pixel_offset * world_per_px)
            }
            crate::core::resources::CameraKind::Orthographic => {
                let world_height = 2.0 / proj_yy;
                let world_per_px = world_height / viewport_h;
                Some(pixel_offset * world_per_px)
            }
        }
    }

    fn push_segment_quad(
        &mut self,
        start: Vec3,
        end: Vec3,
        line_dir: Vec3,
        perp: Vec3,
        start_half_width: f32,
        end_half_width: f32,
        color: Vec4,
    ) {
        // Extend each segment along its tangent to create square caps and
        // increase overlap between adjacent segments.
        let s_center = start - line_dir * start_half_width;
        let e_center = end + line_dir * end_half_width;
        let s_left = s_center + perp * start_half_width;
        let s_right = s_center - perp * start_half_width;
        let e_left = e_center + perp * end_half_width;
        let e_right = e_center - perp * end_half_width;

        self.push_vertex(s_left, color);
        self.push_vertex(e_left, color);
        self.push_vertex(e_right, color);

        self.push_vertex(s_left, color);
        self.push_vertex(e_right, color);
        self.push_vertex(s_right, color);
    }

    fn push_vertex(&mut self, position: Vec3, color: Vec4) {
        self.vertices.push(GizmoVertex {
            position,
            _pad: 0.0,
            color,
        });
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if let Some(buffer) = &self.buffer {
            if !self.vertices.is_empty() {
                rpass.set_vertex_buffer(0, buffer.slice(..));
                rpass.draw(0..self.vertices.len() as u32, 0..1);
            }
        }
    }
}
