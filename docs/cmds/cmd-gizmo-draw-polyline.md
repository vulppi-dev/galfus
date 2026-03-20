# CmdGizmoDrawPolyline

Draws a 3D polyline gizmo using a point list (`path`) in world space. Gizmos are cleared every frame.
Gizmos render only to the main color path and do not contribute to emissive/bloom.
Thickness is rendered as a solid triangulated ribbon per segment.

## Arguments

| Field     | Type        | Description                                                   |
| --------- | ----------- | ------------------------------------------------------------- |
| points    | Vec<Vec3>   | Ordered points of the path (at least 2 points)               |
| color     | Vec4        | Color of the polyline (RGBA)                                 |
| closed    | bool        | (Optional) If true, closes the path from last to first       |
| thickness | Option<f32> | (Optional) Screen-space thickness in pixels. `0`/unset keeps thin line. |

## Response

Returns `CmdResultGizmoDraw`:

| Field  | Type | Description                 |
| ------ | ---- | --------------------------- |
| status | u32  | Status code (0 for success) |
