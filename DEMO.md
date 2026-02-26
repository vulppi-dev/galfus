# DEMO.md

## Objetivo

Demos no Vulfram servem para validar, de forma incremental, os principais aspectos da engine (core, render, janela, input, UI, recursos e eventos), sem compromisso de retrocompatibilidade nesta fase experimental.

## Padrões Globais (obrigatórios)

1. Toda janela deve fechar por:
   - evento de fechamento da janela
   - atalho `Ctrl+W`
2. Todo demo deve ter um layer de UI direto com FPS em tempo real visível em tela.
3. Todo demo deve fechar ao pressionar `Escape` (padrão de demos).

## Estrutura Base dos Novos Demos

Cada demo novo deve seguir a mesma espinha dorsal:

1. `setup`:
   - criar janela/realm
   - criar cena mínima
   - registrar UI com FPS
2. `runtime`:
   - loop principal
   - atualização de estado da cena
   - atualização de FPS na UI
3. `input/events`:
   - tratar close event
   - tratar `Ctrl+W`
   - tratar `Escape`
4. `teardown`:
   - encerrar sessão de forma limpa
   - descartar recursos temporários do demo

## Demos

1. **Demo 001 - Bootstrap de Janela e Loop**
   - Escopo: inicialização mínima, frame loop, fechamento por `close event`, `Ctrl+W` e `Escape`, UI layer com FPS.
   - Comandos foco: `window-create`, `window-close`.
   - Subsistemas: lifecycle, window, event queue, UI overlay base.
2. **Demo 002 - Estado de Janela e Cursor**
   - Escopo: medir viewport, alternar estado de janela e cursor.
   - Comandos foco: `window-measurement`, `window-state`, `window-cursor`.
   - Subsistemas: window state/cache/events.
3. **Demo 003 - Realm Lifecycle**
   - Escopo: criar e destruir realms de forma determinística.
   - Comandos foco: `realm-create`, `realm-dispose`.
   - Subsistemas: realm state/graph/report.
4. **Demo 004 - Targets e Layers (base)**
   - Escopo: criar target, organizar layers e descarte.
   - Comandos foco: `target-upsert`, `target-layer-upsert`, `target-layer-dispose`, `target-dispose`.
   - Subsistemas: target graph/resolve/state.
5. **Demo 005 - Câmera e Visibilidade por LayerMask**
   - Escopo: câmera mínima com ordenação e máscara de visibilidade.
   - Comandos foco: `camera-upsert`, `camera-list`, `camera-dispose`.
   - Subsistemas: camera resource + visibilidade.
6. **Demo 006 - Geometria Primitiva + Modelo**
   - Escopo: criar geometria primitiva, instanciar modelo e atualizar pose.
   - Comandos foco: `primitive-geometry-create`, `model-upsert`, `pose-update`, `model-list`, `model-dispose`.
   - Subsistemas: geometry/model + transform pipeline.
7. **Demo 007 - Geometria Custom e Reuso**
   - Escopo: geometria por dados de host e reuso entre múltiplos modelos.
   - Comandos foco: `geometry-upsert`, `geometry-list`, `geometry-dispose`.
   - Subsistemas: vertex allocator/storage.
8. **Demo 008 - Materiais (Standard/PBR)**
   - Escopo: materiais compartilhados, troca dinâmica e fallback.
   - Comandos foco: `material-upsert`, `material-list`, `material-dispose`.
   - Subsistemas: material pipeline/branches.
9. **Demo 009 - Luzes e Sombreamento Básico**
   - Escopo: múltiplas luzes, intensidade e mascaramento por layer.
   - Comandos foco: `light-upsert`, `light-list`, `light-dispose`.
   - Subsistemas: light state + forward lighting.
10. **Demo 010 - Texturas Sólidas e Bind em Target**
   - Escopo: textura de cor sólida e binding em alvos.
   - Comandos foco: `texture-create-solid-color`, `texture-bind-target`, `texture-list`, `texture-dispose`.
   - Subsistemas: texture registry + target bindings.
11. **Demo 011 - Upload Buffer + Texture Async Decode**
   - Escopo: upload binário, decode assíncrono, cancelamento/descarta e fallback.
   - Comandos foco: `texture-create-from-buffer`, `upload-buffer-discard-all`.
   - Subsistemas: buffers, image decode async, eventos de pronto/erro.
12. **Demo 012 - Environment Profile**
   - Escopo: presets de ambiente (clear, skybox, msaa) por layer.
   - Comandos foco: `environment-upsert`, `environment-dispose`.
   - Subsistemas: environment selection por câmera/layer.
13. **Demo 013 - Shadow Map Config**
   - Escopo: configuração global de sombras e validação visual.
   - Comandos foco: `shadow-configure`.
   - Subsistemas: shadow atlas + shadow pass.
14. **Demo 014 - Pós-processamento de Filtros**
   - Escopo: chain de filtros (`filter_*`, `filter_enabled`) e clamp de `outline_threshold`.
   - Comandos foco: `environment-upsert` (bloco `post`).
   - Subsistemas: post pass, outline pass.
15. **Demo 015 - SSAO e Blur**
   - Escopo: oclusão ambiente em profundidade com blur bilateral.
   - Comandos foco: `environment-upsert` (bloco `post.ssao_*`).
   - Subsistemas: `ssao` + `ssao-blur` passes.
16. **Demo 016 - Bloom e Emissive Pipeline**
   - Escopo: threshold/knee/intensity/scatter com emissive forward.
   - Comandos foco: `environment-upsert` (bloco `post.bloom_*`).
   - Subsistemas: bloom pass + compose.
17. **Demo 017 - UI Runtime Base**
   - Escopo: tema, documento, ops e HUD de FPS oficial dos demos.
   - Comandos foco: `ui-theme-define`, `ui-theme-dispose`, `ui-document-create`, `ui-document-set-theme`, `ui-document-set-rect`, `ui-apply-ops`, `ui-document-dispose`.
   - Subsistemas: RealmUI state/render/input bridge.
18. **Demo 018 - UI Introspection, Focus e Debug**
   - Escopo: inspeção de árvore/layout, foco e trace de eventos.
   - Comandos foco: `ui-document-get-tree`, `ui-document-get-layout-rects`, `ui-focus-set`, `ui-focus-get`, `ui-event-trace-set`, `ui-debug-set`.
   - Subsistemas: UI diagnostics + input routing.
19. **Demo 019 - UI Imagens, Clipboard e AccessKit**
   - Escopo: imagens assíncronas na UI, colagem clipboard, screenshot reply e ações de acessibilidade.
   - Comandos foco: `ui-image-create-from-buffer`, `ui-image-dispose`, `ui-clipboard-paste`, `ui-screenshot-reply`, `ui-access-kit-action-request`.
   - Subsistemas: ui image async + host bridge.
20. **Demo 020 - Gizmos e Debug Visual 3D**
   - Escopo: draw calls de linha e AABB para depuração de cena.
   - Comandos foco: `gizmo-draw-line`, `gizmo-draw-aabb`.
   - Subsistemas: gizmo pass.
21. **Demo 021 - RealmPlane, Viewport UI e Raycast**
   - Escopo: UI em plano 3D, viewport embutida e roteamento de ponteiro.
   - Comandos foco: `target-upsert` (`realm-plane`), `target-layer-upsert`, `ui-apply-ops` com `WidgetRealmViewport`.
   - Subsistemas: input raycast/routing + target/ui integração.
22. **Demo 022 - Multi-janela e Composição Multi-target**
   - Escopo: múltiplas janelas e composição de surfaces/realms.
   - Comandos foco: `window-*`, `realm-*`, `target-*` (orquestrados em conjunto).
   - Subsistemas: platform proxy + realm graph compose.
23. **Demo 023 - Áudio: Recursos, Fontes e Listener**
   - Escopo: decode/stream de áudio, listener, fonte espacial e transporte.
   - Comandos foco: `audio-resource-upsert`, `audio-resource-dispose`, `audio-listener-upsert`, `audio-listener-dispose`, `audio-source-upsert`, `audio-source-transport`, `audio-source-dispose`, `audio-state-get`.
   - Subsistemas: audio proxy (desktop/web), eventos de progresso/pronto.
24. **Demo 024 - Input Completo (Keyboard/Mouse/Touch/Gamepad)**
   - Escopo: mapa unificado de input e eventos por dispositivo.
   - Comandos foco: integração com `window-*`, `ui-*`, `pose-update` (reativo a input).
   - Subsistemas: input events, converters, gamepad cache/state/events.
25. **Demo 025 - Diagnóstico, Notificação e Fluxos de Erro**
   - Escopo: política de diagnóstico, notificações e garantia de `SystemEvent::Error`.
   - Comandos foco: `system-diagnostics-set`, `notification-send`.
   - Subsistemas: system diagnostics/notification/error + event pool.
26. **Demo 026 - Render Graph Custom (Passes Conhecidos)**
   - Escopo: pipeline completo com `shadow`, `light-cull`, `skybox`, `forward`, `outline`, `ssao`, `ssao-blur`, `bloom`, `post`, `compose`, `ui`.
   - Comandos foco: combinação de `environment-*`, `shadow-configure`, `target-*`, recursos e UI para validar grafo custom.
   - Subsistemas: render graph/planner/cache/fallback.
27. **Demo 027 - ABI e Filas Host/Core**
   - Escopo: ciclo completo de `send_queue`, `receive_queue`, `receive_events`, coerência de envelopes e respostas.
   - Comandos foco: suíte mista (janela, recursos, UI, áudio) com validação de roundtrip.
   - Subsistemas: cmd flow, response flow, serialization contracts.
28. **Demo 028 - Cenário Full Engine (Stress + Regressão Manual)**
   - Escopo: exercício integrado de todos os sistemas/subsistemas para validar o potencial total da engine.
   - Comandos foco: toda a superfície de `docs/cmds`.
   - Subsistemas: todos (window, realm/target, render, resources, UI, áudio, input, system, profiling).
