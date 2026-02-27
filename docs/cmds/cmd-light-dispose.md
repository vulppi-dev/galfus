# CmdLightDispose

Removes a light source.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| realmId  | u32  | ID of the realm           |
| lightId  | u32  | ID of the light to remove |

## Response

Returns `CmdResultLightDispose`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was removed |
| message | String | Status or error message       |
