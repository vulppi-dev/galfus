# CmdCameraUpsert

Upserts a camera (`Create` or `Update`) in a specific realm.

## Arguments

Accepts one of:

- `CmdCameraCreateArgs`
- `CmdCameraUpdateArgs`

Key fields:

- `realmId` (required)
- `cameraId` (required)
- `transform`, `kind`, `nearFar`, `layerMask`, `order`
- `viewPosition` (optional viewport override)
- `orthoScale` (for orthographic projection)

## Notes

- Camera ownership is realm-scoped (`realmId`).
- Projection is resolved by the core from the effective target/surface size.
- In `TargetLayer`, `cameraId` can override which camera samples a realm.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
