# CmdMaterialUpsert

Upserts a material (`Create` or `Update`) in the universal resource registry.

## Arguments

Accepts one of:

- `CmdMaterialCreateArgs`
- `CmdMaterialUpdateArgs`

Key fields:

- `materialId` (required)
- `kind` (`standard` or `pbr`)
- `options` (material-specific payload):
    - `topology` (optional): `point-list`, `line-list`, `triangle-list` (default)
    - `polygonMode` (optional): `fill` (default), `line`, `point`
    - (other standard/pbr specific properties)

## Notes

- Material ownership is global (window-agnostic).
- Models reference materials by logical `materialId`.
- Missing textures referenced by material keep fallback sampling behavior.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
