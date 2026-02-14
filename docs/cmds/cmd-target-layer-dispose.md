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

If layer key (`realmId`, `targetId`) is not found:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "command"`).
