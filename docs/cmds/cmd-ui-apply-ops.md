# CmdUiApplyOps

Aplica um lote de operações declarativas em um `UiDocument`.

## Arguments

| Field      | Type   | Description |
| ---------- | ------ | ----------- |
| documentId | u32    | Logical UI document ID |
| version    | u64    | Versão monotônica do lote |
| ops        | UiOp[] | Operações (`add/remove/clear/set/move`) |

## UiOp

- `add { parent?, node, index? }`
- `remove { nodeId }`
- `clear { parent? }`
- `set { nodeId, props }`
- `move { nodeId, newParent?, index? }`

## UiNode

| Field       | Type           | Description |
| ----------- | -------------- | ----------- |
| id          | u32            | Node ID |
| kind        | UiNodeKind     | Tipo do nó |
| props       | UiNodeProps    | Payload do tipo |
| tooltip     | Option<String> | Tooltip |
| contextMenu | Option<Vec<String>> | Menu de contexto declarativo |
| anim        | Option<UiAnim> | Animações (`opacity`, `translateY`) |
| display     | Option<bool>   | Remove do layout/hit-test quando `false` |
| visible     | Option<bool>   | Invisível e não interativo quando `false` |
| opacity     | Option<f32>    | Multiplicador de opacidade |
| zIndex      | Option<i32>    | Z-order no documento |

## UiNodeKind

`container`, `window`, `panel`, `split-pane`, `area`, `frame`, `scroll-area`, `grid`,
`popup`, `tooltip`, `modal`, `resize`, `scene`, `canvas`, `text`, `rich-text`, `link`,
`hyperlink`, `button`, `checkbox`, `radio`, `selectable-label`, `toggle`, `slider`,
`drag-value`, `progress-bar`, `combo-box`, `menu-button`, `collapsing-header`,
`image-button`, `spinner`, `text-edit`, `input`, `image`, `widget-realm-viewport`,
`separator`, `spacer`.

Consulte detalhes de `UiNodeProps` em `docs/ui/WIDGETS.md` e `src/core/ui/types.rs`.

## Response

Returns `CmdResultUiApplyOps`:

| Field   | Type        | Description |
| ------- | ----------- | ----------- |
| success | bool        | Lote aplicado com sucesso |
| message | String      | Status ou erro |
| version | Option<u64> | Versão persistida (ou atual em caso de erro de versão) |

## Validation

- `documentId` deve existir.
- `version` deve ser maior que a versão atual do documento.
- Em caso de erro em qualquer op:
  - core faz rollback do lote no documento;
  - resposta retorna `success = false`.
