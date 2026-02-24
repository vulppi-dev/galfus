# CmdGizmoDrawAabb

Draws a 3D Axis-Aligned Bounding Box (AABB) gizmo. Gizmos are cleared every frame.
Gizmos render only to the main color path and do not contribute to emissive/bloom.

## Arguments

| Field | Type | Description                   |
| ----- | ---- | ----------------------------- |
| min   | Vec3 | Minimum corner (x, y, z)      |
| max   | Vec3 | Maximum corner (x, y, z)      |
| color | Vec4 | Color of the box lines (RGBA) |

## Response

Returns `CmdResultGizmoDraw`:

| Field  | Type | Description                 |
| ------ | ---- | --------------------------- |
| status | u32  | Status code (0 for success) |
