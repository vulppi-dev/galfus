# CmdRenderGraphDispose

Disposes a render graph resource from the global catalog.

## Arguments

| Field         | Type | Description |
| ------------- | ---- | ----------- |
| renderGraphId | u32  | Logical render graph ID |

## Response

Returns `CmdResultRenderGraphDispose`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Whether dispose succeeded |
| message | String | Status or error message |

## Validation Rules

- Core-reserved default graph IDs cannot be disposed.
- A graph bound to one or more realms cannot be disposed.

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "render-graph"`).
