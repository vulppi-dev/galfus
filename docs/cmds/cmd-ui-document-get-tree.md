# CmdUiDocumentGetTree

Retorna a árvore declarativa do documento UI.

## Arguments

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| documentId | u32  | Logical UI document ID |

## Response

Returns `CmdResultUiDocumentGetTree`:

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| success    | bool | Consulta bem-sucedida |
| message    | String | Status ou erro |
| documentId | Option<u32> | Documento consultado |
| version    | Option<u64> | Versão atual do documento |
| rootNodes  | UiDocumentTreeNode[] | Árvore ordenada por raiz |

`UiDocumentTreeNode`:

- `nodeId`
- `kind`
- `zIndex`
- `children[]`
