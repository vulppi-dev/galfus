# CmdUiFocusGet

Retorna o estado de foco UI por janela.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| windowId | Option<u32> | Filtra por janela (opcional) |

## Response

Returns `CmdResultUiFocusGet`:

| Field   | Type | Description |
| ------- | ---- | ----------- |
| success | bool | Consulta bem-sucedida |
| message | String | Status ou erro |
| entries | UiFocusEntry[] | Entradas de foco |

`UiFocusEntry`:

- `windowId`
- `realmId`
- `documentId`
- `nodeId`
