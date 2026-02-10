# CmdUiDebugSet

Toggles UI debug overlays (bounds/ids) and basic profiling in the UI pass.

## Arguments

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| enabled    | bool | Master enable for UI debug overlays |
| showBounds | bool | Draws node bounds rectangles |
| showIds    | bool | Draws node ids on top of bounds |
| showProfile | bool | Shows UI layout/tessellation timings |

## Response

Returns `CmdResultUiDebugSet`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether debug was updated    |
| message | String | Status or error message      |
