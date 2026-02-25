# CmdModelUpsert

Upserts a model (`Create` or `Update`) in a specific realm.

## Arguments

Accepts one of:

- `CmdModelCreateArgs`
- `CmdModelUpdateArgs`

Key fields:

- `realmId` (required)
- `modelId` (required)
- `geometryId`, `materialId`
- `transform`, `layerMask`
- `castShadow`, `receiveShadow`, `castOutline`, `outlineColor`

## Notes

- Model ownership is realm-scoped (`realmId`).
- `geometryId` and `materialId` are global logical IDs.
- Missing referenced resources use fallback behavior until available.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
