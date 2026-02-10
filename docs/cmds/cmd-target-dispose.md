# CmdTargetDispose

Removes a logical target from the auto-graph maps.

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
