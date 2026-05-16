# RealmUI — Visão Geral

`RealmUI` é o runtime declarativo de UI da engine. O host envia comandos e ops (`CmdUi*`) e o core:

1. mantém estado de documentos e temas;
2. converte entrada (ponteiro/teclado) para `egui::Event`;
3. renderiza malhas UI no pass de UI;
4. devolve eventos UI/sistema para o host.

## Estrutura Semântica

- `UiTheme`:
  - recurso reutilizável (dados visuais + fontes).
- `UiDocument`:
  - árvore de nós UI vinculada a um `realmId`.
- `UiNode`:
  - unidade declarativa (`kind + props`) aplicada por `CmdUiApplyOps`.
- `UiImage`:
  - recurso de imagem assíncrono consumido por `Image/ImageButton`.

## Integração com Targets/Layers

- `UiImageSource::Target(targetId)` permite sample de alvo externo.
- `UiNodeKind::Image` com `UiImageSource::Target` permite renderizar target em nó de UI.
- UI em plano 3D com raycast/hit usa targets e materiais, sem target kind legado.

## Referência

- Runtime: [`RUNTIME.md`](RUNTIME.md)
- Widgets: [`WIDGETS.md`](WIDGETS.md)
- Eventos: [`EVENTS.md`](EVENTS.md)
- Painter: [`PAINTER.md`](PAINTER.md)
- Limitações: [`LIMITATIONS.md`](LIMITATIONS.md)
- Comandos: `docs/cmds/cmd-ui-*.md`
