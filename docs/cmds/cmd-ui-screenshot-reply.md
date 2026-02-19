# CmdUiScreenshotReply

Entrega ao core a imagem solicitada por `UiScreenshotRequest`.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| windowId | u32  | Janela de origem |
| realmId  | Option<u32> | Realm alvo (opcional; usa foco se ausente) |
| width    | u32  | Largura em pixels |
| height   | u32  | Altura em pixels |
| rgba     | bytes | Buffer RGBA8 (`width * height * 4`) |

## Response

Returns `CmdResultUiInputEvent`:

| Field   | Type | Description |
| ------- | ---- | ----------- |
| success | bool | Evento entregue |
| message | String | Status ou erro |
