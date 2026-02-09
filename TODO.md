# TODO — Render Architecture Replace (Realm/Surface/RealmGraph)

> Checklist incremental e detalhado para migrar do modelo window-centric para Realm/Surface/RealmGraph (projeto experimental, sem retrocompatibilidade por enquanto).

**Fase 0 — Preparacao e alinhamento tecnico**
- [x] Mapear o fluxo atual de render por window e os pontos de acoplamento (`WindowState.render_state`, `render_frames`, `CmdRenderGraphSet`).
- [x] Definir a separacao de estados: `EngineState` (global), `WindowState` (janela), `InputsState` (teclado/ponteiro/gamepad), `UniversalState` (tabelas globais), `RealmState` (por realm).
- [x] Definir o contrato interno de `Realm`, `Surface`, `Present` e `Connector` (campos minimos, defaults e lifecycle).
- [x] Definir como os IDs logicos novos (`RealmId`, `SurfaceId`, `ConnectorId`, `PresentId`) aparecem no core (tabelas + generation).
- [x] Definir a politica de buffering do `Surface` (min 2 imagens, prev/current) e como expor `PreviousFrame`.
- [x] Definir regras de composicao multi-janela: mesma `Surface` em N windows no mesmo frame.
- [x] Definir regras do `rect` (connector) com comportamento tipo `position: fixed` do CSS.
- [x] Validar o impacto no profiling e GPU timestamps com execucao multi-realm.
- [x] Atualizar docs de arquitetura (planejamento) antes de iniciar implementacao pesada.

**Fase A — Infraestrutura base (sem retrocompatibilidade)**
- [x] Criar `RealmTable` com generation e estados essenciais (kind, output_surface, render_graph, flags).
- [x] Criar `SurfaceTable` com generation, `kind` (onscreen/offscreen), size/format/alpha/msaa policies e buffering.
- [x] Criar `PresentTable` com mapping `windowId -> surfaceId` e defaults.
- [x] Criar `ConnectorTable` com campos minimos: `connectorId`, `kind`, `sourceSurfaceId`, `rect`, `zIndex`, `blendMode`, `clip`, `inputFlags`.
- [x] Implementar APIs basicas das tabelas (alloc/get/remove).
- [x] Realocar estado do sistema de audio para o `UniversalState` e bindings por `RealmState` (sem mexer em detalhes internos).
- [x] Migrar bind de audio para `realmId` em vez de `windowId` (listener e source).
- [x] Introduzir `Realm` default por window (criado junto com `CmdWindowCreate`).
- [x] Criar `Surface` onscreen por window (virtual swapchain) e registrar em `Present`.
- [x] Substituir `CmdRenderGraphSet` por `CmdRenderGraph3DSet` (remover comando antigo).
- [x] Criar o comando de RenderGraph para Realm 2D (par 2D do `CmdRenderGraph3DSet`).
- [x] Atualizar docs dos comandos de RenderGraph para refletir a divisao 3D/2D.
- [x] Atualizar docs de `RENDER-GRAPH` para esclarecer que e intra-Realm.

**Fase B — RealmGraph minimo (composicao visual basica)**
- [x] Implementar `RealmGraphPlanner` com build de grafo a partir de `Connectors` + `Presents`.
- [x] Implementar deteccao de ciclo (SCC) e politica de corte de edges soft.
- [x] Implementar cache `LastGoodSurface` e `FallbackSurface` para edges cortadas.
- [x] Definir politica `Hard` vs `Soft` para edges (present roots = hard, demais = soft default).
- [x] Implementar execucao topologica de Realms (sem input routing).
- [x] Implementar composicao por `Connector` no Realm consumidor (pass interno) respeitando `zIndex`, `blendMode` e `clip`.
- [x] Implementar regras de ordenacao: `zIndex` ordena layers quando varios connectors apontam para a mesma `Surface`.
- [x] Implementar `rect` com regras tipo `position: fixed` (coordenadas relativas a window/viewport + clip por width).
- [x] Implementar alinhamento por height e clip por width ao desenhar a `Surface` em cada viewport.
- [x] Implementar resolucao automatica de MSAA para surfaces sampleaveis.
- [x] Criar conversoes automaticas de formato/alpha/size no compositor do Realm.
- [x] Garantir que o `Surface` seja sempre renderavel e sampleavel.
- [x] Atualizar docs de arquitetura com o fluxo `RealmGraph` e ciclo-break.

**Fase C — Input routing (hit-test e foco)**
- [x] Implementar hit-test de `ViewportConnector` (rect + z-order + clip).
- [x] Implementar retrace/raycast de `PlaneConnector` (screen -> UV -> local).
- [x] Implementar pointer capture (down->up) e focus lock por connector.
- [x] Implementar `eventTrace` e metadata: `Window -> Realm -> Connector -> Realm`.
- [x] Garantir que o trace suporte multiplas janelas com a mesma `Surface`.
- [x] Garantir compatibilidade com eventos atuais (nao quebrar payloads existentes).
- [x] Atualizar docs de eventos para incluir routing opcional.

**Fase D — Qualidade, performance e observabilidade**
- [x] Implementar guard contra self-sample no frame atual (permitir apenas `PreviousFrame`).
- [x] Implementar guard de no-progress (dirty ping-pong) com teto de iteracoes por frame.
- [x] Implementar throttling por Realm (importance, cachePolicy).
- [x] Consolidar conversoes automaticas no compositor (colorspace/alpha/size/resolve).
- [x] Implementar `FrameReport` com ordem de execucao, edges cortadas e surfaces em cache/fallback.
- [x] Garantir que nenhum Realm bloqueia o tick por readback (pipeline N-1).
- [x] Atualizar docs de profiling para incluir informacoes de RealmGraph/FrameReport.

**Fase E — Cleanups e consolidacao**
- [x] Remover caminhos window-centric antigos (execucao direta por window) quando RealmGraph estiver estavel.
- [x] Auditar fallback e caches (evitar leaks, respeitar politicas de dispose).
- [x] Revisar validacao do RenderGraph intra-Realm para cobrir recursos/inputs/outputs de fato.
- [x] Atualizar `docs/ARCH.md`, `docs/OVERVIEW.md` e `docs/API.md` com a nova arquitetura.
- [x] Atualizar exemplos/demos para usar os novos comandos quando existirem.
- [x] Rodar `scripts/check.sh` antes de finalizar a fase.

**Fase F — Auto-Graph (targets/realms por binds, sem graphs no host)**
- [ ] Definir contrato dos mapas logicos do host: `RealmMap`, `TargetMap`, `BindMap` (binds com `layout`).
- [ ] Definir tipos de `TargetKind`: `Window`, `ViewportEmbed`, `PanelEmbed`, `Texture`.
- [ ] Definir estrutura de `BindLayout`: `rect`, `zIndex`, `clip`, `inputFlags`.
- [ ] Definir regras deterministicas para inferir parent automaticamente (sem map explicito).
- [ ] Definir politica de desempate para binds conflitantes (multi-window, multi-parent).
- [ ] Documentar o fluxo: host -> maps -> auto TargetGraph + RealmGraph.

**Fase G — TargetGraph interno (cache + diffs)**
- [ ] Criar `TargetId`, `TargetState` e `TargetTable` no core.
- [ ] Criar `TargetGraphPlan` (arvore de targets + ordem de composicao).
- [ ] Implementar cache `TargetGraphCache` com hash dos binds/targets.
- [ ] Implementar diff incremental: detectar add/remove/update de bind e aplicar delta.
- [ ] Implementar invalidação parcial por target afetado (auto-balanceamento).
- [ ] Expor `FrameReport` com stats do TargetGraph (nodes/edges/updates).

**Fase H — Resolucao automatica de Surface/Present/Connector**
- [ ] Resolver Surface automaticamente a partir de `Bind(realm -> target)`.
- [ ] Criar/atualizar `Connector` quando target != root.
- [ ] Criar/atualizar `Present` quando target == `Window`.
- [ ] Atualizar layout do connector a partir do `BindLayout`.
- [ ] Garantir dispose automatico ao remover binds/targets.
- [ ] Consolidar politicas de tamanho/format/alpha/msaa no target.

**Fase I — Input routing por TargetGraph**
- [ ] Resolver hit-test pela arvore de targets (zIndex/clip/rect).
- [ ] Mapear `targetId` no `eventTrace` junto com realm/connector.
- [ ] Suportar `ViewportEmbed` (2D) e `PanelEmbed` (UI) com regras de layout.
- [ ] Suportar `PlaneConnector` (3D) com raycast quando aplicavel.
- [ ] Garantir capture/focus por target (nao apenas connector).

**Fase J — Demos e validacao**
- [ ] Criar demo que usa apenas binds/logical maps (sem comandos de graph).
- [ ] Exercitar: Window + ViewportEmbed + PanelEmbed + Texture.
- [ ] Exercitar multi-window com mesmo target e binds conflitantes.
- [ ] Exercitar ciclo e self-sample com auto-cut.
- [ ] Atualizar docs com exemplos do fluxo auto-graph.
