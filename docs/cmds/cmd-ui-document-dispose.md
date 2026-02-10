# CmdUiDocumentDispose

Disposes a UI document.

## Arguments

| Field      | Type | Description            |
| ---------- | ---- | ---------------------- |
| documentId | u32  | Logical UI document ID |

## Response

Returns `CmdResultUiDocumentDispose`:

| Field   | Type | Description                     |
| ------- | ---- | ------------------------------- |
| success | bool | Whether the document was disposed |
| message | String | Status or error message      |
