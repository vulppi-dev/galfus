# CmdUiAccessKitActionRequest

Bridge para ações de acessibilidade AccessKit enviadas pelo host.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| windowId | u32  | Janela de origem |
| realmId  | Option<u32> | Realm alvo (opcional) |
| action   | String | Ação serializada pelo host |

## Response

Returns `CmdResultUiInputEvent`:

| Field   | Type | Description |
| ------- | ---- | ----------- |
| success | bool | `false` no fallback atual |
| message | String | Mensagem diagnóstica |

## Notes

No runtime atual, este comando é fallback e emite `SystemEvent::Error` (`scope = "ui-input"`).
