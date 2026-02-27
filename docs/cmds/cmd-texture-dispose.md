# CmdTextureDispose

Removes a texture resource or target-bound texture.

If materials reference this texture, they will render with fallbacks until a
texture with the same ID is created again.

## Arguments

| Field     | Type | Description                 |
| --------- | ---- | --------------------------- |
| textureId | u32  | ID of the texture to remove |

## Response

Returns `CmdResultTextureDispose`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was removed |
| message | String | Status or error message         |
