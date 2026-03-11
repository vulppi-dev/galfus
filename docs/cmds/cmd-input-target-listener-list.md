# CmdInputTargetListenerList

Lists registered input target listeners.

## Arguments

| Field    | Type        | Description |
| -------- | ----------- | ----------- |
| targetId | Option<u64> | Optional filter by target |

## Response

Returns `CmdResultInputTargetListenerList`:

| Field     | Type                          | Description |
| --------- | ----------------------------- | ----------- |
| success   | bool                          | Whether listing succeeded |
| message   | String                        | Status message |
| listeners | InputTargetListenerSnapshot[] | Matching listeners |

`InputTargetListenerSnapshot` fields:
- `listenerId`, `targetId`, `enabled`
- `events`, `samplePercent`

## Notes

- If `targetId` is omitted, all listeners are returned.
