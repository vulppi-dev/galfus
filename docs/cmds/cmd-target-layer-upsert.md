# CmdTargetLayerUpsert

Upserts a layer binding from one realm into one target.

## Arguments

| Field | Type | Description |
| --- | --- | --- |
| realmId | u32 | Logical realm id |
| targetId | u64 | Logical target id |
| layout | TargetLayerLayout | Rect and composition properties |
| cameraId | Option<u32> | Optional camera override for 3D sampling |
| environmentId | Option<u32> | Optional environment override |

## TargetLayerLayout

| Field | Type | Description |
| --- | --- | --- |
| left | DimensionValue | Layer X offset |
| top | DimensionValue | Layer Y offset |
| width | DimensionValue | Layer width |
| height | DimensionValue | Layer height |
| enabled | bool | Defaults to `true` |
| opacity | f32 | Defaults to `1.0` |
| zIndex | i32 | Visual order key |
| blendMode | u32 | Blend selector |
| clip | Option<Vec4> | Optional clip rect |

## DimensionValue

`DimensionValue` is serialized as:

```json
{ "unit": "px" | "percent" | "character" | "display", "value": <number> }
```

## Validation

- `layout.width` and `layout.height` must resolve to `> 0`.
- IDs are late-bound; upsert can succeed before all referenced objects exist.

## Response

`CmdResultTargetLayerUpsert { success, message }`.

## Determinism

- Layers are ordered by `zIndex`.
- Equal `zIndex` uses stable deterministic tie-break from layer key order.
