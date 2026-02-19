# RealmUI Runtime

## Fluxo de Estado

1. Host envia `CmdUiDocumentCreate`, `CmdUiThemeDefine`, `CmdUiApplyOps`.
2. Core atualiza `UiState` (`documents`, `themes`, `images`, foco e caches).
3. Pass UI executa layout/paint e gera draw data.
4. `PlatformOutput` do egui vira:
   - comandos internos de janela/cursor/IME;
   - eventos para host (`UiOpenUrl`, clipboard, screenshot request, viewport sync).

## Entrada

- Pointer/keyboard chegam do subsistema de input.
- Roteamento passa por `target layer` e resolve realm/document foco.
- Para `RealmPlane`/`WidgetRealmViewport`, a posição é transformada para coordenadas locais.

## Saída

- Eventos de widget: `EngineEvent::Ui(UiEvent)`.
- Eventos de sistema UI: `EngineEvent::System(SystemEvent::Ui*)`.
- Falhas diagnósticas: `SystemEvent::Error` também é emitido.

## Recursos Assíncronos

- `CmdUiImageCreateFromBuffer` inicia decode assíncrono.
- Progresso por eventos:
  - `UiImageProcessingStarted`
  - `UiImageProcessingProgress`
  - `UiImageProcessingFinished`
  - `UiImageReady`
- `CmdUiImageDispose` cancela decode pendente e descarta resultado.

## Foco

- Foco é mapeado por `windowId + realmId + documentId + nodeId`.
- Comandos:
  - `CmdUiFocusSet`
  - `CmdUiFocusGet`

## Trace de Input

- `CmdUiEventTraceSet` controla:
  - nível (`off/errors/basic/full`);
  - sampling (`0..100`).
- Trace inclui hops realm/target/layer para depuração de roteamento.
