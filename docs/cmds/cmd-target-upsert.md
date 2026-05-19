# CmdTargetUpsert

Upserts a logical target used by frame composition.

## Arguments

| Field | Type | Description |
| --- | --- | --- |
| targetId | u64 | Logical target id |
| kind | TargetKind | `window` or `texture` |
| windowId | Option<u32> | Required for `window`; forbidden for `texture` |
| size | Option<UVec2> | Optional declared size for `texture`; ignored for `window` |
| formatPolicy | Option<SurfaceFormatDto> | Optional surface/texture format policy |
| alphaPolicy | Option<SurfaceAlphaModeDto> | Optional alpha composition mode |
| msaaSamples | Option<u32> | Optional sample count hint |

## Validation

- `kind=window` requires `windowId`.
- `kind=texture` rejects `windowId`.
- `size` is accepted only for `kind=texture`.
- texture `size` is clamped to at least `1x1`.

## Response

`CmdResultTargetUpsert { success, message }`.

## Notes

- references are late-bound; target upsert does not require all dependencies to exist immediately.
- target clear/composition is resolved during frame execution.
