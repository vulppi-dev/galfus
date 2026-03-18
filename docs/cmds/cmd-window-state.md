# CmdWindowState

Aplica e consulta estado/propriedades de janela em um único comando.

Agrupa:
- título
- estado da janela (`minimized`, `maximized`, `windowed`, `fullscreen`, `windowed-fullscreen`)
- ícone por `bufferId`
- decorations / resizable
- ações de foco e atenção
- consulta de `state` / `decorations` / `resizable`

## Platform Notes

- **Desktop:** suporte completo para patch + get.
- **WASM:** suporte parcial:
  - mutações (`title`, `state`, `decorations`, `resizable`, `action`, `iconBufferId`) não são suportadas;
  - getters suportados: `getState` e `getResizable`;
  - `getDecorations` retorna `None` (não aplicável em canvas).

## Arguments

| Field          | Type                    | Description |
| -------------- | ----------------------- | ----------- |
| windowId       | u32                     | ID da janela |
| title          | Option<String>          | Novo título (opcional) |
| state          | Option<EngineWindowState> | Novo estado da janela (opcional) |
| iconBufferId   | Option<u64>             | Buffer de imagem para ícone (opcional) |
| decorations    | Option<bool>            | Habilita/desabilita decorations (opcional) |
| resizable      | Option<bool>            | Habilita/desabilita resize (opcional) |
| action         | Option<WindowStateAction> | Ação opcional: `focus`, `request-attention` |
| attentionType  | Option<UserAttentionType> | Tipo para `request-attention`: `critical`, `informational` |
| getState       | bool                    | Inclui `state` na resposta |
| getDecorations | bool                    | Inclui `decorations` na resposta |
| getResizable   | bool                    | Inclui `resizable` na resposta |

## Response

Retorna `CmdResultWindowState`:

| Field       | Type                    | Description |
| ----------- | ----------------------- | ----------- |
| success     | bool                    | Se o comando foi aplicado com sucesso |
| message     | String                  | Status ou erro |
| state       | Option<EngineWindowState> | Estado atual/atualizado quando solicitado |
| decorations | Option<bool>            | Valor atual/atualizado quando solicitado |
| resizable   | Option<bool>            | Valor atual/atualizado quando solicitado |

## Notes

- Campos de patch são opcionais: envie apenas o que deseja alterar.
- `iconBufferId` consome o upload (one-shot) e exige `uploadType = \"image-data\"`.
- `fullscreen` exige modo exclusivo; se indisponível, o comando retorna `success=false` e o core emite `SystemEvent::Error`.
- Mudanças de ciclo de vida disparam `WindowEvent::OnStateChange`.
