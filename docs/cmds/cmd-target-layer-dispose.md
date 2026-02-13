# CmdTargetLayerDispose

Removes a logical layer between a realm and a target.

## Arguments

| Field    | Type | Description        |
| -------- | ---- | ------------------ |
| realmId  | u32  | Logical realm ID   |
| targetId | u64  | Logical target ID  |

## Response

Returns `CmdResultTargetLayerDispose`:

| Field   | Type   | Description                 |
| ------- | ------ | --------------------------- |
| success | bool   | Whether the layer was disposed |
| message | String | Status or error message     |
