# SYSTEMS.md — Mapa de Sistemas da Vulfram Engine

Este documento lista os sistemas atuais do core (`src/core`) agrupados por domínio semântico, com responsabilidades e módulos principais.

## 1) Runtime Core, ABI e Orquestração

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| ABI pública `vulfram_*` | Ponto de entrada/saída do host para inicialização, envio de comandos, leitura de eventos/respostas, upload e tick | `src/core/mod.rs`, `src/core/lifecycle.rs`, `src/core/queue.rs`, `src/core/tick.rs`, `src/core/profiling/cmd.rs` |
| Singleton do engine | Guardar instância global do `EngineState` e garantir acesso coordenado | `src/core/singleton.rs` |
| Estado global (`EngineState`) | Estado central de janelas, device/queue, recursos, grafos, filas, profiling e caches | `src/core/state.rs` |
| Ciclo de frame | Processar comandos, input, auto-graphs, render, eventos e profiling por frame | `src/core/tick.rs` |
| Barramento de comandos/respostas/eventos | Decodificação MessagePack, dispatch de comandos e enfileiramento de respostas/eventos | `src/core/cmd.rs`, `src/core/queue.rs` |

## 2) Plataforma e Janela (Host-facing Runtime Windowing)

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Abstração de plataforma | Alias/tipos de plataforma para desktop/browser | `src/core/platform/mod.rs`, `src/core/platform/native.rs`, `src/core/platform/web.rs` |
| Proxies desktop/browser | Integração de eventos e APIs específicas por plataforma | `src/core/platforms/desktop/*`, `src/core/platforms/browser/*`, `src/core/platforms/mod.rs` |
| Gerência de janelas | Criar/atualizar/fechar janelas, manter `WindowState`, cache e mapping de IDs | `src/core/window/state.rs`, `src/core/window/cache.rs`, `src/core/window/mod.rs` |
| Comandos de janela | Criação nativa/wasm, cursor, atenção, propriedades, decoração, close | `src/core/window/cmd/create*.rs`, `src/core/window/cmd/cursor.rs`, `src/core/window/cmd/attention.rs`, `src/core/window/cmd/properties.rs`, `src/core/window/cmd/decorations.rs`, `src/core/window/cmd/close.rs` |
| Eventos de janela | Eventos emitidos ao host (create/resize/close/focus etc.) | `src/core/window/events.rs` |

## 3) Input, Ponteiro, Teclado e Roteamento Inter-Realm

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Estado/caches de input | Cache de estado e deduplicação em desktop | `src/core/input/state.rs`, `src/core/input/cache.rs` |
| Conversão de eventos de plataforma | Traduzir eventos de Winit/Web para formato interno | `src/core/input/events/converters.rs`, `src/core/input/events/*` |
| Eventos de input | Tipos de evento internos (pointer/keyboard/modifiers/scroll/touch) | `src/core/input/events/common.rs`, `src/core/input/events/pointer.rs`, `src/core/input/events/keyboard.rs` |
| Raycast 3D | Interseção ponteiro->câmera->mundo para seleção/hit | `src/core/input/raycast.rs` |
| Routing de ponteiro | Propagação por camadas realm/target/connector com UV e trace | `src/core/input/routing.rs` |

## 4) Gamepad

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Polling e conversão gamepad | Integrar Gilrs (desktop) e Web Gamepad (wasm), converter para eventos internos | `src/core/gamepad/mod.rs`, `src/core/gamepad/converters.rs` |
| Estado e cache gamepad | Estado por dispositivo, filtros de ruído e deduplicação | `src/core/gamepad/state.rs`, `src/core/gamepad/cache.rs` |
| Eventos gamepad | Eventos `OnConnect/OnDisconnect/OnButton/OnAxis` | `src/core/gamepad/events.rs` |

## 5) Realm, Surface, Present, Connector e Planejamento de Grafo

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Tabelas universais (realm domain) | Guardar realms, surfaces, presents, connectors, auto-links, cache de superfícies e routing state | `src/core/realm/state.rs` |
| Comandos de realm | Create/dispose e tipos de realm (`TwoD`, `ThreeD`) | `src/core/realm/cmd/realm.rs`, `src/core/realm/cmd/types.rs`, `src/core/realm/cmd/mod.rs` |
| RealmGraph planner | Construir ordem de execução de realms, tratar cortes de ciclo e plano final | `src/core/realm/graph.rs` |
| Frame report e diagnósticos | Relatório por frame (ordem, cut edges, target stats, throttling etc.) | `src/core/realm/report.rs` |

## 6) Targets, Layers e Auto-Graph

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Modelo de target/layer | Tipos de target (`window`, `texture`, `realm-plane`, `widget-realm-viewport`) e layout (`DimensionValue`) | `src/core/target/state.rs` |
| Comandos de target/layer | Upsert/dispose de targets e target layers | `src/core/target/cmd.rs` |
| TargetGraph | Planejamento/ordenação de dependências entre targets | `src/core/target/graph.rs` |
| Hash/diff de target graph | Invalidação incremental e detecção de mudanças | `src/core/target/graph_hash.rs` |
| Resolução auto-link | Criar/atualizar surfaces/presents/connectors a partir de target layers | `src/core/target/resolve.rs` |

## 7) Recursos de Cena e Render (Resource Domain)

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Câmera | Specs/records e comandos create/update/dispose/list | `src/core/resources/camera/spec.rs`, `src/core/resources/camera/cmd.rs`, `src/core/resources/camera/mod.rs` |
| Modelo | Dados de modelo e comandos upsert/dispose/list | `src/core/resources/model/spec.rs`, `src/core/resources/model/cmd.rs`, `src/core/resources/model/mod.rs` |
| Luz | Dados de luz e comandos upsert/dispose/list | `src/core/resources/light/spec.rs`, `src/core/resources/light/cmd.rs`, `src/core/resources/light/mod.rs` |
| Geometria | Geometrias primitivas e utilitários (AABB/frustum/generators) | `src/core/resources/geometry/*` |
| Material | Standard/PBR, tipos e handlers de comando | `src/core/resources/material/spec.rs`, `src/core/resources/material/cmd/*`, `src/core/resources/material/mod.rs` |
| Textura | Criação/atlas/bind target/decode assíncrono e utilitários de comando | `src/core/resources/texture/spec.rs`, `src/core/resources/texture/cmd/*`, `src/core/resources/texture/async_decode.rs`, `src/core/resources/texture/forward_atlas.rs` |
| Environment | Ambiente por ID (MSAA, skybox, post, clear_color) e comandos | `src/core/resources/environment/spec.rs`, `src/core/resources/environment/cmd.rs`, `src/core/resources/environment/mod.rs` |
| Shadow resources | Atlas/configuração e integração com luzes/sombras | `src/core/resources/shadow/atlas.rs`, `src/core/resources/shadow/cmd.rs`, `src/core/resources/shadow/mod.rs` |
| Vertex allocator | Arena/alocação/binds/cache de buffers de vértice | `src/core/resources/vertex/*` |
| Storage/list/common/uniform | Tabelas, listagem, utilitários e UBOs | `src/core/resources/storage.rs`, `src/core/resources/list.rs`, `src/core/resources/common.rs`, `src/core/resources/uniform.rs`, `src/core/resources/spec.rs`, `src/core/resources/mod.rs` |

## 8) Pipeline de Renderização

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Orquestrador de render | Execução do frame render completo, composição por realms/surfaces/targets | `src/core/render/mod.rs` |
| Render graph (por realm) | Plano de passes (`shadow`, `skybox`, `forward`, `post`, `compose`, `ui` etc.) | `src/core/render/graph.rs` |
| Compose realm graph | Coleta de views de surface, composição de connectors e cache de fallback | `src/core/render/realm_graph.rs` |
| Render state | Estado por janela: cena, caches, bindings, pipelines, atlas, UI renderers | `src/core/render/state/mod.rs`, `src/core/render/state/scene.rs`, `src/core/render/state/lifecycle.rs` |
| Inicialização de sistemas de render | Pipelines/layouts/fallbacks/samplers/uniform buffers | `src/core/render/state/init/*`, `src/core/render/state/library.rs` |
| Preparação de frame | Atualização de bindings, materiais, luzes e estruturas de draw | `src/core/render/state/prepare/*`, `src/core/render/state/binding.rs`, `src/core/render/state/light.rs` |
| Coleta e cache de draw | Coletor de draw calls, cache de pipelines/shaders | `src/core/render/state/collector.rs`, `src/core/render/cache.rs` |
| Skinning | Sistema de skinning e buffers correspondentes | `src/core/render/state/skinning.rs` |
| Gizmos | Pipeline de desenho de gizmos/linhas/AABB | `src/core/render/gizmos/mod.rs`, `src/core/render/gizmos/gizmo.wgsl` |

## 9) Passes de Render (Detalhado)

| Pass | Responsabilidade | Módulos principais |
|---|---|---|
| Shadow | Atualização/render de sombras no atlas | `src/core/render/passes/shadow/mod.rs`, `src/core/render/passes/shadow/shadow.wgsl` |
| Skybox | Render de skybox (none/procedural/cubemap) | `src/core/render/passes/skybox/mod.rs`, `src/core/render/passes/skybox/skybox.wgsl` |
| Light Culling | Culling de luzes por tile/frustum para shading | `src/core/render/passes/light_cull/mod.rs`, `src/core/render/passes/light_cull/light_cull.wgsl` |
| Forward | Render principal (branches Standard/PBR), mesh collection/draw | `src/core/render/passes/forward/mod.rs`, `src/core/render/passes/forward/collector.rs`, `src/core/render/passes/forward/draw.rs`, `src/core/render/passes/forward/branches/*` |
| SSAO | Ambient occlusion e variantes MSAA/blur | `src/core/render/passes/ssao/mod.rs`, `src/core/render/passes/ssao/*.wgsl` |
| Outline | Pass de contorno | `src/core/render/passes/outline/mod.rs`, `src/core/render/passes/outline/outline.wgsl` |
| Bloom | Cadeia de bloom e composições | `src/core/render/passes/bloom/mod.rs`, `src/core/render/passes/bloom/bloom.wgsl` |
| Post | Tonemap/filtros finais | `src/core/render/passes/post/mod.rs`, `src/core/render/passes/post/post.wgsl` |
| Compose | Blit/composição final para o target de saída | `src/core/render/passes/compose/mod.rs`, `src/core/render/passes/compose/overlay.rs`, `src/core/render/passes/compose/compose.wgsl` |
| UI | Render egui em realms 2D e composição com texturas externas | `src/core/render/passes/ui/mod.rs` |

## 10) UI Realm (egui Integration)

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Modelo declarativo UI | Definir nós, props, layout, animação e ops | `src/core/ui/types.rs` |
| Estado UI | Estado por realm/document, buffers de input, animações, themes, imagens, foco | `src/core/ui/state.rs` |
| Comandos UI | Document/theme/image/debug + apply ops versionado | `src/core/ui/cmd/document.rs`, `src/core/ui/cmd/theme.rs`, `src/core/ui/cmd/image.rs`, `src/core/ui/cmd/debug.rs`, `src/core/ui/cmd/mod.rs` |
| Input para UI | Converter eventos do core para `egui::Event` e focar realm/documento | `src/core/ui/input.rs` |
| Render de documentos UI | Montar widgets/layout e emitir `UiEvent` | `src/core/ui/render.rs`, `src/core/ui/events.rs` |
| Renderer de UI | Pipeline WGSL de UI, texturas internas/externas e draw de primitives | `src/core/ui/renderer/mod.rs`, `src/core/ui/renderer/pipeline.rs`, `src/core/ui/renderer/textures.rs`, `src/core/ui/ui.wgsl` |
| Decode assíncrono de UI images | Jobs/worker para decodificação e eventos de progresso/resultado | `src/core/ui/image_async.rs` |

## 11) Áudio

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| API de áudio | Comandos para listener, resources e source playback | `src/core/audio/cmd.rs` |
| Proxy de backend | Interface comum de backend de áudio | `src/core/audio/proxy.rs`, `src/core/audio/mod.rs` |
| Backend desktop | Implementação com Kira | `src/core/audio/kira.rs` |
| Backend browser | Implementação com WebAudio | `src/core/audio/webaudio.rs` |

## 12) Buffers, Upload e Imagem

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Upload buffers | Entrada one-shot de blobs binários do host (geometria/textura/etc.) | `src/core/buffers/state.rs`, `src/core/buffers/cmd.rs`, `src/core/buffers/mod.rs` |
| Decodificação de imagem | Representação e utilitários de buffers de imagem | `src/core/image.rs` |

## 13) Sistema de Eventos Genéricos (não-render)

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Eventos de sistema | Eventos transversais (ex.: erro, progresso de jobs) | `src/core/system/events.rs`, `src/core/system/mod.rs` |
| Notificações | Integração de notificações (desktop/browser quando suportado) | `src/core/system/notification.rs` |

## 14) Profiling e Telemetria

| Sistema | Responsabilidade | Módulos principais |
|---|---|---|
| Profiling de tick/frame | Métricas de CPU/FPS, serialização, render timings | `src/core/profiling/state.rs`, `src/core/profiling/mod.rs` |
| Profiling GPU | Query timestamp e agregação por pass | `src/core/profiling/gpu.rs` |
| Export de profiling | Comando ABI para obter snapshot de profiling | `src/core/profiling/cmd.rs` |

## 15) Relações-Chave Entre Sistemas (Fluxo Real)

1. Host envia comandos/upload via ABI (`queue` + `buffers`).
2. `cmd.rs` aplica mutações no `EngineState` (resources, targets/layers, realms, UI, janela, áudio).
3. `tick.rs` processa input/gamepad, roteamento e auto-graphs (`target -> realm/surface/present/connector`).
4. `render/mod.rs` executa `RealmGraph` + `RenderGraph` por realm e compõe em surfaces/targets.
5. `pass_ui` integra egui no realm 2D e usa texturas externas de targets (`widget-realm-viewport`, `texture`, etc.).
6. Eventos/respostas/profiling retornam ao host por MessagePack.

## 16) Observações de Escopo Atual

- O core está organizado em domínio de runtime, domínio de world graph (realm/target), domínio de recursos e domínio de passes.
- IDs lógicos são controlados pelo host; o core gerencia estado físico/caches/pools.
- `TargetLayer` é a unidade de ligação de realm->target para composição e roteamento.
- A integração UI usa egui com árvore declarativa própria do core, além de renderer WGSL dedicado.
