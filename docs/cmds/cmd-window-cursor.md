# CmdWindowCursor

Configura o cursor do mouse da janela (não confundir com eventos de pointer genérico).

Agrupa:
- visibilidade do cursor
- modo de grab/lock
- ícone do cursor

## Platform Notes

- **WASM:** Não suportado (retorna `success=false` com mensagem).

## Arguments

| Field    | Type                  | Description |
| -------- | --------------------- | ----------- |
| windowId | u32                   | ID da janela |
| visible  | Option<bool>          | Mostra/oculta cursor (opcional) |
| mode     | Option<CursorGrabMode> | Grab mode (opcional): `none`, `confined`, `locked` |
| icon     | Option<CursorIcon>    | Ícone do cursor (opcional) |

### CursorIcon (Enum)

- `default`
- `context-menu`
- `help`
- `pointer`
- `progress`
- `wait`
- `cell`
- `crosshair`
- `text`
- `vertical-text`
- `alias`
- `copy`
- `move`
- `no-drop`
- `not-allowed`
- `grab`
- `grabbing`
- `e-resize`
- `n-resize`
- `ne-resize`
- `nw-resize`
- `s-resize`
- `se-resize`
- `sw-resize`
- `w-resize`
- `ew-resize`
- `ns-resize`
- `nesw-resize`
- `nwse-resize`
- `col-resize`
- `row-resize`
- `all-scroll`
- `zoom-in`
- `zoom-out`

## Response

Retorna `CmdResultWindowCursor`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Se o comando foi aplicado com sucesso |
| message | String | Status ou erro |

## Notes

- Todos os campos são opcionais: envie apenas o subset a atualizar.
- Quando `icon` é enviado via comando do host, o ícone vira override persistente para a janela e tem prioridade sobre atualizações de cursor vindas do pipeline de UI.
