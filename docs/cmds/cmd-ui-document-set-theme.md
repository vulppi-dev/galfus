# CmdUiDocumentSetTheme

Updates the theme of a UI document.

## Arguments

| Field      | Type        | Description            |
| ---------- | ----------- | ---------------------- |
| documentId | u32         | Logical UI document ID |
| themeId    | Option<u32> | Theme ID to apply      |

## Response

Returns `CmdResultUiDocumentSetTheme`:

| Field   | Type | Description                     |
| ------- | ---- | ------------------------------- |
| success | bool | Whether the theme was updated   |
| message | String | Status or error message      |
