# CmdUiApplyOps

Applies a batch of UI ops to a document.

## Arguments

| Field      | Type   | Description                        |
| ---------- | ------ | ---------------------------------- |
| documentId | u32    | Logical UI document ID             |
| version    | u64    | Monotonic version for this batch   |
| ops        | UiOp[] | Operations to apply                |

### UiOp

Supported ops:

- `add { parent?, node, index? }`
- `remove { nodeId }`
- `clear { parent? }`
- `set { nodeId, props }`
- `move { nodeId, newParent?, index? }`

## Response

Returns `CmdResultUiApplyOps`:

| Field   | Type        | Description                     |
| ------- | ----------- | ------------------------------- |
| success | bool        | Whether ops were applied        |
| message | String      | Status or error message         |
| version | Option<u64> | Stored version                  |
