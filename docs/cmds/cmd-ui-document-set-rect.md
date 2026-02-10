# CmdUiDocumentSetRect

Updates the rect of a UI document.

## Arguments

| Field      | Type | Description            |
| ---------- | ---- | ---------------------- |
| documentId | u32  | Logical UI document ID |
| rect       | Vec4 | Rect (x, y, w, h)      |

## Response

Returns `CmdResultUiDocumentSetRect`:

| Field   | Type | Description                     |
| ------- | ---- | ------------------------------- |
| success | bool | Whether the rect was updated    |
| message | String | Status or error message      |
