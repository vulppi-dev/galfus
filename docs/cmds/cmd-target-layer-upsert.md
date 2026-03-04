# CmdTargetLayerUpsert

Upserts a logical layer between a realm and a target.

## Arguments

| Field         | Type              | Description |
| ------------- | ----------------- | ----------- |
| realmId       | u32               | Logical realm ID |
| targetId      | u64               | Logical target ID |
| layout        | TargetLayerLayout | Layout and routing configuration |
| cameraId      | Option<u32>       | Optional camera override for Realm3D sampling |
| environmentId | Option<u32>       | Optional environment profile override |

### TargetLayerLayout

| Field      | Type        | Description                         |
| ---------- | ----------- | ----------------------------------- |
| left       | DimensionValue | X position |
| top        | DimensionValue | Y position |
| width      | DimensionValue | Layer width |
| height     | DimensionValue | Layer height |
| zIndex     | i32         | Layer order                         |
| blendMode  | u32         | Blend mode selector                 |
| clip       | Option<Vec4>| Optional clip rect                  |

### DimensionValue

`DimensionValue` is encoded as:

```json
{ "unit": "px" | "percent" | "character" | "display", "value": <number> }
```

- `px`: absolute pixels
- `percent`: axis-relative percentage (`left/width` use realm width, `top/height` use realm height)
- `character`: text width unit (`ch`)
- `display`: display unit (`dp`, resolved as `value * 4px`)

Pointer routing mode is inferred internally by the core from target kind.
For `window` and `widget-realm-viewport` layers sourced from `Realm3D`,
raycast routing is enabled automatically.

Composition behavior for connector-like targets (`window` non-host layer and `realm-plane`):

- Follows **cover fit** semantics.
- Height is fitted first.
- Width overflow is center-clipped (no stretch/deformation).

Environment selection priority per layer:
1. `environmentId` from the layer (if set);
2. current default environment profile;
3. core internal hardcoded default.

Camera selection for `realm kind = 3d`:
1. `cameraId` from the layer (if set);
2. first available camera in the realm;
3. if no camera exists, layer renders only background (`clearColor`/skybox).

## Response

Returns `CmdResultTargetLayerUpsert`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the layer was upserted |
| message | String | Status or error message      |

## Validation Rules

- `layout.width` and `layout.height` must resolve to values `> 0`.
- `realmId`, `targetId`, `cameraId`, and `environmentId` are late-bound references:
  command upsert is accepted even when referenced IDs do not exist yet.
  Resolution occurs automatically when those IDs become available.

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "command"`).
