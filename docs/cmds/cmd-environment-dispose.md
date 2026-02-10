# CmdEnvironmentDispose

Resets the environment configuration to defaults for a window.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultEnvironment`:

| Field   | Type   | Description                         |
| ------- | ------ | ----------------------------------- |
| success | bool   | Whether the environment was disposed |
| message | String | Status or error message              |
