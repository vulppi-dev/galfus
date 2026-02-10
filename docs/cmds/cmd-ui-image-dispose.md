# CmdUiImageDispose

Disposes a UI image resource. If decode is pending, it is canceled and the result is discarded.

## Arguments

| Field   | Type | Description          |
| ------- | ---- | -------------------- |
| imageId | u32  | Logical UI image ID  |

## Response

Returns `CmdResultUiImageDispose`:

| Field   | Type | Description                      |
| ------- | ---- | -------------------------------- |
| success | bool | Whether the image was disposed   |
| message | String | Status or error message       |
