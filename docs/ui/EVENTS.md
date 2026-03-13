# RealmUI Events

## Eventos de Widget (`EngineEvent::Ui`)

`UiEventKind`:

- `click`
- `double-click`
- `pressed`
- `released`
- `hover-enter`
- `hover-leave`
- `changed`
- `change-commit`
- `focus`
- `blur`
- `submit`
- `anim-complete`

Payload (`UiEvent`):

- `realmId`
- `documentId`
- `nodeId`
- `kind`
- `label?`

## Eventos de Sistema UI (`EngineEvent::System`)

- Clipboard:
  - `UiClipboardSetText`
  - `UiClipboardRequestCopy`
  - `UiClipboardRequestCut`
  - `UiClipboardRequestPaste`
- URL:
  - `UiOpenUrl`
- Screenshot:
  - `UiScreenshotRequest`
- Multi-viewport:
  - `UiViewportSync`
  - `UiViewportCommand`
  - `UiViewportFallbackEmbedded`
- Async image:
  - `UiImageProcessingStarted`
  - `UiImageProcessingProgress`
  - `UiImageProcessingFinished`
  - `UiImageReady`
- Diagnóstico:
  - `Error`
- Input targeting:
  - `InputTargetListenerEvent` (inclui posições e dimensões de janela/target quando disponíveis)

## Comandos de Entrada para Eventos UI

- `CmdUiClipboardPaste`
- `CmdUiScreenshotReply`
- `CmdUiAccessKitActionRequest` (fallback de acessibilidade)
