# CmdTargetBindDispose

Removes a logical bind between a realm and a target.

## Arguments

| Field    | Type | Description        |
| -------- | ---- | ------------------ |
| realmId  | u32  | Logical realm ID   |
| targetId | u64  | Logical target ID  |

## Response

Returns `CmdResultTargetBindDispose`:

| Field   | Type   | Description                 |
| ------- | ------ | --------------------------- |
| success | bool   | Whether the bind was disposed |
| message | String | Status or error message     |
