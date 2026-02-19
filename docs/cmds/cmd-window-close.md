# CmdWindowClose

Closes an existing window and cleans up its resources.

Dispose semantics:
- Removes the window and direct runtime dependencies tied to that `windowId`:
  - `target` entries with matching `windowId`
  - associated `targetLayer` links
  - auto-links (`surface/present/connector`) created from those layers
  - direct input focus/capture bindings for that window
- Indirect dependencies continue via fallback where applicable.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| windowId | u32  | ID of the window to close |

## Response

Returns `CmdResultWindowClose`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the window was closed |
| message | String | Status or error message       |
