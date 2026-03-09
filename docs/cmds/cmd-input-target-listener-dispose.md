# CmdInputTargetListenerDispose

Disposes a previously registered input listener by `listenerId`.

## Arguments

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| listenerId | u64  | Logical listener ID |

## Response

Returns `CmdResultSimple`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Always `true` (idempotent semantics) |
| message | String | Disposed status or no-op status |

## Notes

- Dispose is idempotent.
- Missing listener returns a no-op message, not an error.
