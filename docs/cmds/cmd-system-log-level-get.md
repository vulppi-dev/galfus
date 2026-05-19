# CmdSystemLogLevelGet

Returns the current runtime log threshold for the engine instance.

## Arguments

No arguments.

## Response

Returns `CmdResultSystemLogLevelGet`:

| Field | Type | Description |
| --- | --- | --- |
| success | bool | Whether the read succeeded |
| message | String | Status message |
| currentLevel | `trace \| debug \| info \| warn \| error` | Current threshold used by log emission filter |
