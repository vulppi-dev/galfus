# CmdUiClipboardPaste

Entrega texto de paste do host para o realm UI focado da janela.

## Arguments

| Field    | Type   | Description |
| -------- | ------ | ----------- |
| windowId | u32    | Janela de destino |
| text     | String | Texto colado |

## Response

Returns `CmdResultUiInputEvent`:

| Field   | Type | Description |
| ------- | ---- | ----------- |
| success | bool | Evento entregue |
| message | String | Status ou erro |
