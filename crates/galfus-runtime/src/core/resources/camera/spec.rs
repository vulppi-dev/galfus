use bytemuck::{Pod, Zeroable};
pub use galfus_render::{RenderTarget, ensure_render_target};
use glam::{Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CameraKind {
    Perspective = 0,
    Orthographic,
}

impl CameraKind {
    pub fn to_u32(self) -> u32 {
        match self {
            CameraKind::Perspective => 0,
            CameraKind::Orthographic => 1,
        }
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(CameraKind::Perspective),
            1 => Some(CameraKind::Orthographic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct CameraComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub up: Vec4,
    pub near_far: Vec2,
    pub kind_flags: UVec2,

    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
}

impl CameraComponent {
    /// Create from raw input data
    ///
    /// For both Perspective and Orthographic cameras, uses `projection_size` (width, height) to calculate aspect ratio.
    /// For Orthographic cameras, `ortho_scale` defines the vertical span of the view.
    pub fn new(
        transform: Mat4,
        kind: CameraKind,
        flags: u32,
        near_far: Vec2,
        projection_size: (u32, u32),
        ortho_scale: f32,
    ) -> Self {
        let position = transform.w_axis.truncate();
        let rotation = Quat::from_mat4(&transform);
        let direction = rotation * Vec3::NEG_Z;
        let up = rotation * Vec3::Y;

        let view = Mat4::look_to_rh(position, direction, up);

        let aspect_ratio = projection_size.0 as f32 / projection_size.1 as f32;

        let projection = match kind {
            CameraKind::Perspective => {
                let fov_y = 45.0_f32.to_radians();
                // Reverse-Z projection: keep depth clear at 0.0 and depth compare Greater.
                Mat4::perspective_rh(fov_y, aspect_ratio, near_far.y, near_far.x)
            }
            CameraKind::Orthographic => {
                let half_height = ortho_scale / 2.0;
                let half_width = half_height * aspect_ratio;
                Mat4::orthographic_rh(
                    -half_width,  // left
                    half_width,   // right
                    -half_height, // bottom
                    half_height,  // top
                    near_far.y,
                    near_far.x,
                )
            }
        };

        let view_projection = projection * view;

        Self {
            position: position.extend(1.0),
            direction: direction.extend(0.0),
            up: up.extend(0.0),
            near_far,
            kind_flags: UVec2::new(kind.to_u32(), flags),
            projection,
            view,
            view_projection,
        }
    }

    /// Update from raw input data
    pub fn update(
        &mut self,
        transform: Option<Mat4>,
        kind: Option<CameraKind>,
        flags: Option<u32>,
        near_far: Option<Vec2>,
        projection_size: (u32, u32),
        ortho_scale: f32,
    ) {
        let transform = transform.unwrap_or_else(|| {
            let pos = self.position.truncate();
            let dir = self.direction.truncate();
            let up = self.up.truncate();
            Mat4::look_to_rh(pos, dir, up).inverse()
        });

        let kind = kind.unwrap_or_else(|| {
            CameraKind::from_u32(self.kind_flags.x).unwrap_or(CameraKind::Perspective)
        });

        let flags = flags.unwrap_or(self.kind_flags.y);
        let near_far = near_far.unwrap_or(self.near_far);

        *self = Self::new(
            transform,
            kind,
            flags,
            near_far,
            projection_size,
            ortho_scale,
        );
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ViewValue {
    Relative(f32),
    Absolute(u32),
}

impl ViewValue {
    pub fn resolve(&self, total: u32) -> u32 {
        match *self {
            ViewValue::Relative(value) => {
                let value = (value * total as f32).round() as u32;
                value.max(1)
            }
            ViewValue::Absolute(value) => value.max(1),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewAnchor {
    pub x: ViewValue,
    pub y: ViewValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewSize {
    pub width: ViewValue,
    pub height: ViewValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewPosition {
    pub anchor: ViewAnchor,
    pub size: ViewSize,
}

impl ViewPosition {
    pub fn resolve_size(&self, total_width: u32, total_height: u32) -> (u32, u32) {
        let width = self.size.width.resolve(total_width).max(1);
        let height = self.size.height.resolve(total_height).max(1);
        (width, height)
    }

    pub fn resolve_position(&self, total_width: u32, total_height: u32) -> (u32, u32) {
        let x = match self.anchor.x {
            ViewValue::Relative(value) => (value * total_width as f32).round() as u32,
            ViewValue::Absolute(value) => value,
        };
        let y = match self.anchor.y {
            ViewValue::Relative(value) => (value * total_height as f32).round() as u32,
            ViewValue::Absolute(value) => value,
        };
        (x, y)
    }
}

#[derive(Debug, Clone)]
pub struct CameraNode {
    pub label: Option<String>,
    pub data: CameraComponent,
    pub layer_mask: u32,
    pub order: i32,
    pub ortho_scale: f32,
    pub view_position: Option<ViewPosition>,
}

impl CameraNode {
    pub fn to_render_record(&self) -> CameraRecord {
        CameraRecord::new(
            self.label.clone(),
            self.data,
            self.layer_mask,
            self.order,
            self.view_position.clone(),
            self.ortho_scale,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CameraRecord {
    pub label: Option<String>,
    pub data: CameraComponent,
    pub layer_mask: u32,
    pub order: i32,
    pub is_dirty: bool,
    pub ortho_scale: f32,
    pub render_target: Option<RenderTarget>,
    pub emissive_target: Option<RenderTarget>,
    pub post_target: Option<RenderTarget>,
    pub outline_target: Option<RenderTarget>,
    pub ssao_target: Option<RenderTarget>,
    pub ssao_blur_target: Option<RenderTarget>,
    pub bloom_target: Option<RenderTarget>,
    pub bloom_chain: [Option<RenderTarget>; 4],
    pub forward_depth_target: Option<RenderTarget>,
    pub forward_msaa_target: Option<RenderTarget>,
    pub forward_emissive_msaa_target: Option<RenderTarget>,
    pub history0_target: Option<RenderTarget>,
    pub history1_target: Option<RenderTarget>,
    pub history_valid: bool,
    pub history_idle_frames: u32,
    pub view_position: Option<ViewPosition>,
    pub last_projection_size: UVec2,
}

impl CameraRecord {
    pub fn new(
        label: Option<String>,
        data: CameraComponent,
        layer_mask: u32,
        order: i32,
        view_position: Option<ViewPosition>,
        ortho_scale: f32,
    ) -> Self {
        Self {
            label,
            data,
            layer_mask,
            order,
            is_dirty: true,
            ortho_scale,
            render_target: None,
            emissive_target: None,
            post_target: None,
            outline_target: None,
            ssao_target: None,
            ssao_blur_target: None,
            bloom_target: None,
            bloom_chain: [None, None, None, None],
            forward_depth_target: None,
            forward_msaa_target: None,
            forward_emissive_msaa_target: None,
            history0_target: None,
            history1_target: None,
            history_valid: false,
            history_idle_frames: 0,
            view_position,
            last_projection_size: UVec2::ZERO,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn effective_target_size(&self, surface_size: UVec2) -> UVec2 {
        let (width, height) = self
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_size(surface_size.x, surface_size.y))
            .unwrap_or((surface_size.x, surface_size.y));
        UVec2::new(width.max(1), height.max(1))
    }

    // Render targets are managed via ensure_render_target helper.
}
