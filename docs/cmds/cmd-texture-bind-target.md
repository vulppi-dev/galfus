# CmdTextureBindTarget

Binds a `textureId` to a `TargetId`, allowing materials to sample a target surface directly when resolved.

## Arguments

| Field     | Type     | Description                     |
| --------- | -------- | ------------------------------- |
| textureId | u32      | Logical texture ID              |
| targetId  | u64      | Target ID reference (late-bound) |
| label     | Option<String> | Optional label for listing |

## Response

Returns `CmdResultTextureBindTarget`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the bind applied |
| message | String | Status or error message  |

## Notes

- `targetId` is a late-bound reference; bind upsert does not fail when the target is missing.
- Sampling becomes active once the referenced target exists and resolves to a texture-producing bind path.
