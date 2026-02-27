# CmdTextureBindTarget

Binds a `textureId` to a `TargetId` of kind `texture`, allowing materials to sample a target surface directly.

## Arguments

| Field     | Type     | Description                     |
| --------- | -------- | ------------------------------- |
| textureId | u32      | Logical texture ID              |
| targetId  | u64      | Target ID (must be `texture`)   |
| label     | Option<String> | Optional label for listing |

## Response

Returns `CmdResultTextureBindTarget`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the bind applied |
| message | String | Status or error message  |
