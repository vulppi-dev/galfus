# CmdGeometryUpsert

Upserts a geometry (`Create` or `Update`) in the universal resource registry.

## Arguments

Accepts one of:

- `CmdGeometryCreateArgs`
- `CmdGeometryUpdateArgs`

Related command:

- `CmdPrimitiveGeometryCreate` for built-in primitives.

## Notes

- Geometry ownership is global (window-agnostic).
- Models reference geometries by logical `geometryId`.
- GPU allocation/packing is handled internally by the core.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
