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
| display | Option<bool> | If false, skips layout + hit-test |
| visible | Option<bool> | If false, invisible and non-interactive |
| opacity | Option<f32> | Opacity multiplier (0..1) |
| zIndex | Option<i32> | Z-order inside the document |
| anim | Option<UiAnim> | Optional animation payload |

### UiNodeKind

`container`, `text`, `button`, `input`, `image`, `separator`, `spacer`

### UiNodeProps (MVP)

- `container { layout, padding?, size?, scrollX?, scrollY? }`
- `text { text, size?, color? }`
- `button { label, enabled? }`
- `input { value, placeholder?, enabled? }`
- `image { source, size? }`
- `separator`
- `spacer { width?, height? }`

### UiLayout

Fields:

- `direction`: `row`, `row-reverse`, `column`, `column-reverse`, `grid`
- `align`: `start`, `center`, `end`, `stretch`
- `justify`: `start`, `center`, `end`, `stretch`
- `gap`: f32
- `columns`: Option<u32> (grid only)
- `wrap`: bool
- `wrapLimit`: Option<f32>

### UiAnim

Optional animation payload on `UiNode`:

- `opacity { from, to, durationMs, easing? }`
- `translateY { from, to, durationMs, easing? }`

`easing` supports `linear` or `ease-in-out`.

### UiImageSource

- `ui-image { content: u32 }` (refere a `UiImageId`)
- `target { content: u64 }` (refere a `TargetId`)

## Response

Returns `CmdResultUiApplyOps`:

| Field   | Type        | Description                     |
| ------- | ----------- | ------------------------------- |
| success | bool        | Whether ops were applied        |
| message | String      | Status or error message         |
| version | Option<u64> | Stored version                  |
