# CmdModelDispose

Removes a model instance.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| realmId  | u32  | ID of the realm           |
| modelId  | u32  | ID of the model to remove |

## Response

Returns `CmdResultModelDispose`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was removed |
| message | String | Status or error message       |
