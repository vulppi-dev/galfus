# CmdInputTargetListenerUpsert

Creates or updates an input listener bound to a logical target.

## Arguments

| Field         | Type                        | Description |
| ------------- | --------------------------- | ----------- |
| listenerId    | u64                         | Logical listener ID (host-managed) |
| targetId      | u64                         | Logical target ID |
| windowId      | Option<u32>                 | Optional window ownership hint |
| enabled       | bool                        | Enables/disables listener dispatch |
| events        | String[]                    | Event filter list. Empty means all mapped events for target |
| scope         | TargetListenerScope         | `"target"` or `"target-and-descendants"` |
| throttleMs    | u32                         | Reserved for throttling policy |
| samplePercent | u8                          | Sampling percentage `[0,100]` |

## Response

Returns `CmdResultSimple`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Always `true` for accepted command |
| message | String | Status message |

## Notes

- Upsert replaces previous config for the same `listenerId`.
- `samplePercent` is clamped to `[0,100]`.
- Host is responsible for logical ID validity/uniqueness.
