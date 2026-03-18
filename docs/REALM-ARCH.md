# Realm/Surface/RealmGraph — Planejamento de Base

Este documento consolida os itens da Fase 0 do replace de arquitetura: contratos, IDs e regras
fundamentais para Realm/Surface/RealmGraph.

## 1. Contratos internos (campos, defaults e lifecycle)

- `Realm`
  - `kind`: `ThreeD` ou `TwoD`.
  - `output_surface`: `SurfaceId` principal do realm.
  - `render_graph_id`: referência lógica para um graph no catálogo global.
  - `flags`: reservado para políticas de execução.
- `Surface`
  - `kind`: `Onscreen` ou `Offscreen`.
  - `size`: dimensão lógica em pixels.
  - `format_policy`: opcional; o core define padrões quando ausente.
  - `alpha_policy`: opcional; o core define padrões quando ausente.
  - `msaa_samples`: opcional; o core resolve quando ausente.
- `Present`
  - `windowId -> surfaceId` (virtual swapchain).
- `Connector`
  - `sourceSurfaceId`
  - `rect`
  - `zIndex`
  - `blendMode`
  - `clip`
  - `inputFlags`

## 2. IDs lógicos e generation

Todas as tabelas (`Realm`, `Surface`, `Connector`, `Present`) usam IDs lógicos com generation.
O objetivo é impedir uso de IDs expirados e garantir remoção segura.

## 3. Buffering de Surface e `PreviousFrame`

Cada `Surface` deve manter pelo menos 2 imagens (current/previous) para permitir:
- leitura do `PreviousFrame` em efeitos dependentes de histórico;
- bloqueio de leitura do output corrente do próprio realm.

## 4. Composição multi-janela

Uma mesma `Surface` pode ser apresentada em múltiplas janelas no mesmo frame:
- ordenação por `zIndex`;
- `rect` com regras tipo `position: fixed` (coordenadas relativas ao viewport);
- `clip` aplicado por conector.

## 5. Regras do `rect` (connector)

`rect` é definido por conector, seguindo comportamento tipo CSS `position: fixed`:
- coordenadas relativas ao viewport da janela;
- clipping e alinhamento por height com corte por width.

## 6. Impacto no profiling e GPU timestamps

O profiling deve refletir execução multi-realm:
- ordem de execução por realm;
- edges cortadas por ciclo;
- surfaces em cache/fallback;
- timestamps por realm/compositor.

## 7. RealmGraph minimo (Fase B)

- O `RealmGraphPlanner` constroi o grafo a partir de `Connectors` e `Presents`.
- Edges `Hard` protegem presents; edges `Soft` podem ser cortadas para quebrar ciclos.
- O plano registra `cut_edges` para diagnostico e cache de `LastGoodSurface`/`FallbackSurface`.
- O compositor do Realm aplica `Connector` por ordem de `zIndex`, com `rect` estilo `position: fixed`,
  alinhamento por altura e clip por largura quando necessario.
- O output do Realm e sempre renderavel/sampleavel (targets float), com composicao convertendo tamanho
  e formato conforme o target final.
