# CmdSystemLogLevelSet

Sets the runtime log threshold for the current engine instance.

Default on engine startup is `info`.

## Arguments

| Field | Type | Description |
| --- | --- | --- |
| level | `trace \| debug \| info \| warn \| error` | Minimum level that will be emitted to host events |

## Behavior

Only logs with level `>= currentLevel` are emitted.

Examples:
- `info` allows: `info`, `warn`, `error`
- `error` allows: `error` only
- `trace` allows all levels

## Response

Returns `CmdResultSystemLogLevelSet`:

| Field | Type | Description |
| --- | --- | --- |
| success | bool | Whether the update succeeded |
| message | String | Status message |
| currentLevel | `trace \| debug \| info \| warn \| error` | Effective level after update |
