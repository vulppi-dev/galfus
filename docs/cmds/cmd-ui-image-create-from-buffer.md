# CmdUiImageCreateFromBuffer

Queues async decode of a UI image from an upload buffer.

## Arguments

| Field    | Type        | Description                 |
| -------- | ----------- | --------------------------- |
| imageId  | u32         | Logical UI image ID         |
| bufferId | u64         | Upload buffer ID            |
| label    | Option<String> | Optional label           |

## Response

Returns `CmdResultUiImageCreateFromBuffer`:

| Field   | Type | Description                      |
| ------- | ---- | -------------------------------- |
| success | bool | Whether the decode was queued    |
| message | String | Status or error message       |
| pending | bool | Whether the decode is pending    |
