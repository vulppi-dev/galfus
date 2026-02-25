# CmdLightUpsert

Upserts a light (`Create` or `Update`) in a specific realm.

## Arguments

Accepts one of:

- `CmdLightCreateArgs`
- `CmdLightUpdateArgs`

Key fields:

- `realmId` (required)
- `lightId` (required)
- `kind`, `position`, `direction`
- `color`, `groundColor`, `intensity`, `range`
- `spotInnerOuter`, `layerMask`, `castShadow`

## Notes

- Light ownership is realm-scoped (`realmId`).
- Light culling and shadow allocation are resolved by the core each frame.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
