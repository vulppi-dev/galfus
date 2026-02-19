# RealmUI Limitações e Fallbacks

## Acessibilidade

- `CmdUiAccessKitActionRequest` atualmente é fallback com erro diagnóstico.
- O comando não executa ação nativa de AccessKit no runtime atual.

## Multi-Viewport

- Quando backend não suporta viewport nativo, o core emite fallback:
  - `UiViewportFallbackEmbedded`
  - `UiViewportCommand` para o host decidir integração.

## Recursos Dependentes de Host

- Clipboard real depende da ponte host <-> sistema operacional.
- Screenshot depende de resposta explícita do host:
  - request: `UiScreenshotRequest`
  - reply: `CmdUiScreenshotReply`

## Comportamentos por Plataforma

- Cursor/IME/foco podem variar por backend (`desktop` x `wasm`).
- Decode assíncrono tenta evitar bloqueio do loop; latência depende do ambiente.
