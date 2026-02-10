# CmdEnvironmentCreate

Creates the environment configuration for a window.

## Arguments

| Field    | Type               | Description |
| -------- | ------------------ | ----------- |
| windowId | u32                | ID of the window |
| config   | EnvironmentConfig  | Environment settings (skybox, lighting, etc.) |

## Response

Returns `CmdResultEnvironment`:

| Field   | Type   | Description                        |
| ------- | ------ | ---------------------------------- |
| success | bool   | Whether the environment was created |
| message | String | Status or error message             |
