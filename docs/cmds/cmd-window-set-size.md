# CmdWindowSetSize

Sets the size of an existing window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).
- **Desktop:** Updates swapchain/surface config and marks window dirty for redraw.

## Arguments

| Field    | Type  | Description         |
| -------- | ----- | ------------------- |
| windowId | u32   | ID of the window    |
| size     | UVec2 | New size dimensions |

## Response

Returns `CmdResultWindowSetSize`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the size was set |
| message | String | Status or error message  |

## Runtime Effects

- Triggers window resize event flow.
- Surface/presentation size is updated by platform event handling.
- Camera projection and render targets are re-synced on the next render tick.
