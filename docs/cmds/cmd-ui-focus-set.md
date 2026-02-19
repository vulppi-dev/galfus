# CmdUiFocusSet

Define foco UI explícito para uma janela.

## Arguments

| Field      | Type | Description |
| ---------- | ---- | ----------- |
| windowId   | u32  | Janela de foco |
| realmId    | u32  | Realm UI alvo |
| documentId | u32  | Documento alvo |
| nodeId     | Option<u32> | Nó alvo (default `0`) |

## Response

Returns `CmdResultUiFocusSet`:

| Field   | Type | Description |
| ------- | ---- | ----------- |
| success | bool | Foco atualizado |
| message | String | Status ou erro |

## Validation

- `realmId` deve existir.
- `documentId` deve existir e pertencer ao `realmId`.
- `nodeId` (quando informado e diferente de `0`) deve existir no documento.
