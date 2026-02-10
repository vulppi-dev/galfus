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

### UiNode

| Field | Type | Description |
| ----- | ---- | ----------- |
| id    | u32  | Node ID     |
| kind  | UiNodeKind | Node kind |
| props | UiNodeProps | Node payload |

### UiNodeKind

`container`, `text`, `button`, `input`, `image`, `separator`, `spacer`

### UiNodeProps (MVP)

- `container { layout, padding?, size?, scrollX?, scrollY? }`
- `text { text, size?, color? }`
- `button { label, enabled? }`
- `input { value, placeholder?, enabled? }`
- `image { imageId, size? }`
- `separator`
- `spacer { width?, height? }`

## Response

Returns `CmdResultUiApplyOps`:

| Field   | Type        | Description                     |
| ------- | ----------- | ------------------------------- |
| success | bool        | Whether ops were applied        |
| message | String      | Status or error message         |
| version | Option<u64> | Stored version                  |
