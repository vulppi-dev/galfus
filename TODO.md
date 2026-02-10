_Sistema de UI renderizado no core, host-driven, usando Realm TwoD + TargetGraph (Surface/Present/Connector internos)._ 

## Fase A — Fundação (Realm UI)
- [x] Definir UI como `Realm` `TwoD` com render graph próprio (pass UI) e fallback.
- [x] Criar estado `UiRealmState` associado ao `realmId` (sem `UiContext` separado).
- [x] Integrar UI com wgpu via draw direto do egui (sem `egui_wgpu`, incompatível com wgpu 28).
- [x] Garantir que o render graph não declare formatos; usar padrões do core (color float `rgba16f`, depth float quando aplicável).
- [x] Definir contrato de recursos UI: `UiTheme`, `UiFont`, `UiImage` como IDs lógicos do host.

## Fase B — Comandos e Modelo de Dados
- [x] `CmdUiThemeDefine/Dispose` + `UiThemeDefined` (cache + versionamento).
- [x] `CmdUiDocumentCreate/Dispose/SetRect/SetTheme` (documento ligado ao `realmId`).
- [x] `CmdUiApplyOps` com versionamento e ops `add/remove/clear/set/move` (sem validação extra de IDs além de consistência interna).
- [x] Definir `UiNodeId` e payloads para widgets MVP.
- [x] `UiImage` decode assíncrono; dispose deve cancelar (ou aguardar) e descartar resultados.

## Fase C — Layout e Widgets MVP
- [x] Widgets MVP: container, text, button, input, image, separator, spacer.
- [x] Layout MVP: row/col/grid, gaps, padding, size (auto/fill/px), align/justify básicos.
- [x] Text/Fonts: fallback, tamanhos por estilo e atlas de glyphs.
- [x] Clipping/Scissor consistente para scrolls e painéis.
- [x] Scroll real com offsets + barras.

## Fase D — Input e Eventos
- [ ] Integrar input routing com `TargetBindLayout.inputFlags` e `eventTrace` (windowId/realmId/targetId/connectorId).
- [ ] Disparar eventos para o egui via proxies de input/janela (core).
- [ ] Ponteiro em UI 2D via hit-box no ambiente 2D.
- [ ] `Panel` é componente 3D (pos/escala/rotação) com UI embutida; aceita alpha e respeita `blendMode`.
- [ ] Ponteiro em UI 3D via trace no `Panel` no ambiente 3D.
- [ ] Padronizar payload de eventos entre conectores (transformação por camada no TargetGraph).
- [ ] `UiEvent` com label + nodeId + realmId.
- [ ] Hit-testing respeitando display/visible/opacity.
- [ ] Focus/keyboard: tab/focus e navegação básica.
- [ ] Z-order interno (overlays/menus) por UiDocument.

## Fase E — Composição (Targets/Surfaces)
- [ ] UI realm pode bindar em targets `window`, `viewport-embed`, `panel-embed` e `texture` via `CmdTargetBindUpsert`.
- [ ] Overlay UI/3D via `zIndex` no `TargetBindLayout` (sem camadas separadas).
- [ ] UI em superfície 3D: render para target `texture` e usar como textura em material/plane.
- [ ] O mesmo realm pode ser usado em múltiplos targets; cada realm mantém seu próprio contexto.
- [ ] Viewport POC: câmera para target texture e UI mostra como imagem.
- [ ] Viewport resize: ajustar target ao tamanho do widget e resolver MSAA automaticamente.

## Fase F — Recursos Avançados
- [ ] Wrap (row/col reversos) com height limitada.
- [ ] Animate: opacity/translateY com easing + `animComplete`.
- [ ] Hot-reload de theme sem recriar o realm.
- [ ] Debug UI: overlay de bounds/ids e profiling básico.
- [ ] Performance: cache de layout + invalidation por dirty flags (inclui cache de tessellation).

## Fase G — Demos e Docs
- [ ] Demo UI com fechamento via Esc.
- [ ] Atualizar demos existentes para continuar funcionando após integração da UI.
- [ ] Refatorar demos de `main.rs` em subarquivos (tamanho).
- [ ] Documentar comandos/ops e exemplos no host.
