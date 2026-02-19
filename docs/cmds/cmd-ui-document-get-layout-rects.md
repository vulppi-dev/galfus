# CmdUiDocumentGetLayoutRects

Retorna os retângulos finais de layout por nó no documento.

## Arguments

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| documentId | u32  | Logical UI document ID |

## Response

Returns `CmdResultUiDocumentGetLayoutRects`:

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| success    | bool | Consulta bem-sucedida |
| message    | String | Status ou erro |
| documentId | Option<u32> | Documento consultado |
| version    | Option<u64> | Versão atual do documento |
| rects      | UiNodeLayoutRect[] | Retângulos por nó |

`UiNodeLayoutRect`:

- `nodeId`
- `rect` (`Vec4: x, y, w, h`)
