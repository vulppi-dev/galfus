# CmdTargetDispose

Removes a logical target from the auto-graph maps.

Dispose semantics:
- Removes direct dependencies of the target:
  - all `targetLayer` entries that reference `targetId`
  - corresponding auto-links (`surface/present/connector`)
  - direct focus references for input routing
- Clears UI external texture references and target size requests tied to the target.

## Arguments

| Field    | Type | Description           |
| -------- | ---- | --------------------- |
| targetId | u64  | Logical target ID     |

## Response

Returns `CmdResultTargetDispose`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the target was disposed |
| message | String | Status or error message         |
