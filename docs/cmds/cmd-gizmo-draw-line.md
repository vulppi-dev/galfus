# CmdGizmoDrawLine

Draws a 3D line gizmo in the scene. Gizmos are cleared every frame.
Gizmos render only to the main color path and do not contribute to emissive/bloom.
Thickness is rendered as a solid triangulated ribbon (not hardware line width).

## Arguments

| Field     | Type        | Description                                                   |
| --------- | ----------- | ------------------------------------------------------------- |
| start     | Vec3        | Starting point of the line                                   |
| end       | Vec3        | Ending point of the line                                     |
| color     | Vec4        | Color of the line (RGBA)                                     |
| thickness | Option<f32> | (Optional) Screen-space thickness in pixels. `0`/unset keeps thin line. |

## Response

Returns `CmdResultGizmoDraw`:

| Field  | Type | Description                 |
| ------ | ---- | --------------------------- |
| status | u32  | Status code (0 for success) |
