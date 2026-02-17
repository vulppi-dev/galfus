# CmdTargetLayerDispose

Removes a logical layer between a realm and a target.

Dispose semantics:
- Removes the direct auto-link generated for (`realmId`, `targetId`).
- If no remaining layer references the target, direct input focus bindings to this target are cleared.

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
