# Vulfram vNext Architecture Replace - Implementation Checklist

## Objetivo
Substituir a arquitetura atual pelo modelo:
- `Realm`
- `Target`
- `Layer`
- `Texture`
- `FrameGraph`
- `RenderInvocation`
- `Graph3D` / `Graph2D`
- `Pass`
- `Shader`
- `Material` / `ShaderMaterial`

Com remoção de conceitos legados públicos:
- `WidgetRealmViewport`
- `RealmPlane`
- `Connector`
- `Present`
- `Surface` (como API pública)
- acoplamentos de realm embutido em target/layer

---

## Fase 0 - Baseline e preparação

### Tasks
- [x] Congelar baseline técnico (branch, commit de referência, nota de rollback).
- [x] Levantar inventário de pontos legados no workspace (`TargetKind`, planner, UI bridge, demos, docs).
- [x] Mapear superfícies públicas afetadas (Rust commands, protocol, TS bindings, docs).
- [x] Definir critério de aceite macro (build, testes alvo, demos mínimas).

### Riscos
- Quebra ampla sem mapa de dependências.

### Validação
- [x] Documento de inventário preenchido.
- [x] Lista de APIs públicas impactadas aprovada.

---

## Fase 1 - Novo contrato de domínio (core types)

### Tasks
- [x] Redefinir `TargetKind` para apenas `Window | Texture`.
- [x] Introduzir/ajustar `Layer` com:
  - [x] `realm_id`
  - [x] `rect`
  - [x] `blend`
  - [x] `opacity`
  - [x] `enabled`
  - [x] `order/priority` (`z_index` neste corte)
- [x] Mover `clear` para `Target` (já consolidado no estado atual; layer/realm sem clear próprio).
- [x] Garantir que `Realm` não retenha resolução/clear.
- [x] Adicionar tipo `RenderInvocation` (runtime/internal):
  - [x] `realm`
  - [x] `target`
  - [x] `layer`
  - [x] `resolved_rect_px`
  - [x] `render_size_px`
  - [x] `frame_id`

### Riscos
- Diferenças semânticas com `TargetLayerLayout` antigo.

### Validação
- [x] Compila `vulfram-realm-core`.
- [x] Testes unitários de serialização/DTO atualizados.
- [x] Nenhuma referência ativa a `WidgetRealmViewport/RealmPlane` em tipos públicos.

---

## Fase 2 - API de comandos e protocolo

### Tasks
- [x] Atualizar `cmd-target-upsert` para aceitar somente `window|texture`.
- [x] Atualizar validações de `windowId`/`size` alinhadas ao novo contrato.
- [ ] Atualizar `cmd-target-layer-upsert` para novo shape de layer/composição.
- [x] Remover regras antigas de viewport/plane/raycast automático por target legado.
- [x] Remover repasse roteado de eventos de ponteiro (target/layer/listener), mantendo apenas stream global de ponteiro.
- [ ] Descontinuar/remover comandos de input target listener e snapshots `pointerTarget*` do host/runtime.
- [ ] Atualizar `vulfram-protocol` e transporte (`transport-*`).
- [ ] Atualizar bindings TS (`packages/engine`, tipos e helpers).

### Riscos
- Incompatibilidade entre runtime e bindings.

### Validação
- [ ] Roundtrip JSON de comandos atualizado.
- [x] Build dos packages TS sem erro de tipo.
- [x] Docs de comandos atualizadas para novo contrato.
- [x] Eventos de ponteiro não incluem mais repasse por target (`positionTarget`, `targetWidth/Height`, `trace`).

---

## Fase 3 - FrameGraph global (targets/texturas)

### Tasks
- [ ] Implementar modelo de `FrameGraph` global com:
  - [ ] targets ativos
  - [ ] layers resolvidas
  - [ ] invocations
  - [ ] texturas produzidas
  - [ ] texturas consumidas
  - [ ] dependências entre targets
- [ ] Substituir planner de auto-links (`Present/Connector`) pelo scheduler de targets.
- [ ] Implementar ordenação topológica de targets.
- [ ] Implementar política de ciclo:
  - [ ] detectar ciclo
  - [ ] usar cache do frame anterior para leituras cíclicas
- [ ] Expor diagnóstico de ordem/ciclos no report interno.

### Riscos
- Deadlock lógico ou ordem não determinística.

### Validação
- [ ] Casos: linear (`Scene -> Blur -> Window`) passa.
- [ ] Casos com ciclo não quebram frame e usam fallback temporal.
- [ ] Ordem é estável entre frames idênticos.

---

## Fase 4 - Resolução de layers e invocações

### Tasks
- [ ] Resolver `rect` por target-size (`pixels`, `%`, `full`, âncoras atuais suportadas).
- [ ] Gerar `RenderInvocation` por ocorrência de layer.
- [ ] Permitir mesmo realm em múltiplos targets/layers no mesmo frame.
- [ ] Garantir isolamento de resolução por invocation.

### Riscos
- Reuso indevido de estado por realm entre invocações de tamanhos distintos.

### Validação
- [ ] Teste com mesmo realm em `1920x1080` e `512x512` no mesmo frame.
- [ ] Snapshot/asserção de `render_size_px` correto por invocation.

---

## Fase 5 - Compositor por target

### Tasks
- [ ] Implementar pipeline conceitual fixo por target:
  - [ ] clear target
  - [ ] render layer image
  - [ ] compor layer no target
- [ ] Aplicar `blend`, `opacity`, `order`, `enabled`.
- [ ] Manter otimizações internas opcionais sem mudar semântica externa.

### Riscos
- Divergência visual em blend/ordem.

### Validação
- [ ] Testes de composição com 2-3 layers sobrepostas.
- [ ] Ordem visual determinística com desempate definido.

---

## Fase 6 - Graph3D e Graph2D separados

### Tasks
- [ ] Criar registries independentes `graph3d` e `graph2d`.
- [ ] Bloquear pass cross-graph.
- [ ] Definir built-ins internos mínimos por graph.
- [ ] Migrar fallback graphs atuais para o novo modelo por realm kind.

### Riscos
- Mistura indevida de recursos 2D/3D.

### Validação
- [ ] Testes de compatibilidade de pass por kind.
- [ ] Realm 3D não aceita pass 2D e vice-versa.

---

## Fase 7 - API de Pass (define/use)

### Tasks
- [ ] Implementar `definePass` com:
  - [ ] `name`
  - [ ] `type` (`screen|draw|compute`)
  - [ ] `input`
  - [ ] `output`
  - [ ] `require`
  - [ ] `params` schema
  - [ ] `shader`
- [ ] Implementar `use(pass, { priority, params, enabled? })` por realm graph.
- [ ] Implementar resolução de ordem por:
  - [ ] dependência `input/output`
  - [ ] `require`
  - [ ] `priority`
  - [ ] nome (desempate estável)
- [ ] Erro para empate perigoso em mesma saída com efeito não comutativo.
- [ ] Política otimista de `input` ausente + `require` bloqueante.

### Riscos
- Ambiguidade de ordem e regressão de efeitos.

### Validação
- [ ] Caso `bright -> glow` ordenado automaticamente.
- [ ] Conflito `fog/vignette` sem prioridade explícita falha corretamente.
- [ ] `require` ausente causa skip de pass.

---

## Fase 8 - Compilação de graph e cache

### Tasks
- [ ] Compilar graph fora do hot path quando houver mudança estrutural.
- [ ] Resolver ping-pong automático em read/write mesmo recurso.
- [ ] Gerar plano executável por invocation.
- [ ] Implementar cache por chave (shader hash/layout/format/sample/etc).
- [ ] No frame quente: apenas bind resources, atualizar params e executar.

### Riscos
- Stutter por recompilação em frame quente.

### Validação
- [ ] Métricas mostram ausência de recompilações contínuas sem mudanças.
- [ ] Reuso de pipeline ativo em cenários estáveis.

---

## Fase 9 - DSL de Shader e geração WGSL físico

### Tasks
- [ ] Definir subset permitido de shader do cliente (funções e structs auxiliares).
- [ ] Bloquear tokens proibidos (`@group`, `@binding`, `@fragment`, etc.).
- [ ] Implementar geração de entrypoints/layout/bindings internos.
- [ ] Expor funções padrão de sample/load por recurso lógico.
- [ ] Mapear `params` para buffer automático por pass-use.

### Riscos
- Segurança/sanidade do parser e geração inválida de WGSL.

### Validação
- [ ] Testes de aceitação/rejeição de shader source.
- [ ] Casos `screen` mínimos compilam e executam.

---

## Fase 10 - ShaderMaterial

### Tasks
- [ ] Introduzir `ShaderMaterial` com base (`standard`/`pbr`).
- [ ] Definir contrato `material(input) -> output`.
- [ ] Integrar no forward sem expor bindings/layout físicos.
- [ ] Garantir coexistência com materiais existentes.

### Riscos
- Explosão de variantes de pipeline.

### Validação
- [ ] Material custom simples (ex: toon) renderiza corretamente.
- [ ] Materiais padrão permanecem estáveis.

---

## Fase 11 - Remoção definitiva do legado

### Tasks
- [ ] Remover `WidgetRealmViewport` e `RealmPlane` de enums/tipos/docs/comandos.
- [ ] Remover caminhos de `Connector/Present/Surface` da API pública.
- [ ] Limpar planner/código morto e testes antigos relacionados.
- [ ] Atualizar demos para o novo fluxo de targets/layers/texturas.

### Riscos
- Referências residuais quebrando build/testes.

### Validação
- [ ] `rg` sem ocorrências legadas em superfície pública.
- [ ] Build completo sem feature toggle de compat.

---

## Fase 12 - UI acoplada ao legado (remoção de acoplamento)

### Tasks
- [ ] Remover dependência da UI dos target kinds legados.
- [ ] Adaptar external textures/UI sampling para `Target Texture` apenas.
- [ ] Manter UI funcional sem conceito de widget viewport especial.

### Riscos
- Regressão em cenários de UI que amostravam target específico.

### Validação
- [ ] Cenários demo de UI renderizando e interagindo normalmente.
- [ ] Sem branches por `WidgetRealmViewport` no runtime.

---

## Fase 13 - Docs e exemplos oficiais

### Tasks
- [ ] Reescrever docs de arquitetura (`REALM-ARCH`, `RENDER-GRAPH`, `API`, cmds).
- [ ] Adicionar exemplos canônicos:
  - [ ] realm em múltiplos targets
  - [ ] pipeline multi-target por textura
  - [ ] passes com `require` e `priority`
- [ ] Atualizar glossário de termos.

### Riscos
- Documentação desatualizada versus implementação.

### Validação
- [ ] Revisão cruzada doc vs comportamento real em demo.

---

## Fase 14 - Hardening e performance

### Tasks
- [ ] Rodar suíte de testes focal + regressão visual mínima.
- [ ] Medir custo de frame e compilações de pipeline.
- [ ] Revisar memory churn (temporários, ping-pong, caches).
- [ ] Auditar logs/erros para mensagens acionáveis.

### Riscos
- Regressão de performance em cenas com muitos layers/passes.

### Validação
- [ ] `cargo test` dos crates críticos verde.
- [ ] Benchmark/smoke sem regressão crítica definida.

---

## Fase 15 - Gate final de release

### Tasks
- [ ] Checklist de breaking changes consolidado.
- [ ] Changelog técnico com migração host.
- [ ] Tag de release vNext e nota de upgrade.

### Riscos
- Consumidores sem trilha de migração.

### Validação
- [ ] Release notes completas e revisadas.

---

## Critérios globais de pronto (DoD)
- [ ] Sem `TargetKind` legado em API pública.
- [ ] FrameGraph ordena targets por dependência de textura.
- [ ] RenderInvocation é a unidade de execução por layer/target.
- [ ] Graph3D/Graph2D separados com passes custom + built-ins.
- [ ] `require/input/output/priority` funcionando deterministicamente.
- [ ] Shader DSL sem bindings físicos expostos.
- [ ] ShaderMaterial funcional sem tomar posse do forward completo.
- [ ] Demos principais migrados para o novo modelo.
- [ ] Documentação oficial alinhada ao comportamento final.

---

## Comandos de validação rápida (referência)
- `cargo test -p vulfram-realm-core`
- `cargo test -p vulfram-render`
- `cargo test -p vulfram-runtime`
- `cargo test -p vulfram-demo`
- `bun run -C packages/engine test` (se existir script)
- `bun run -C apps/demos build` (se existir script)

---

## Notas operacionais
- Estratégia adotada: quebra direta vNext + troca única.
- Durante implementação, remover código morto e símbolos não usados.
- Manter nomes e código técnico em inglês; checklist em PT-BR para fluxo de trabalho.

---

## Decommission obrigatório (aplicar em todas as fases)

### Remoção de crates
- [ ] Remover crates inteiras que não tenham mais papel na arquitetura vNext.
- [ ] Remover referências no `Cargo.toml` raiz e `Cargo.lock`.
- [ ] Remover exports/reexports públicos que apontem para crates extintas.

### Remoção de dependências
- [ ] Remover dependências Rust não usadas de cada crate (`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`).
- [ ] Remover dependências TS/Bun não usadas (`package.json`, lockfile).
- [ ] Remover features/flags antigas sem consumidores.

### Remoção de código e módulos
- [ ] Remover tipos, enums, comandos e handlers legados sem uso.
- [ ] Remover branches condicionais ligadas ao modelo antigo.
- [ ] Remover testes legados que validavam comportamento descontinuado.
- [ ] Remover utilitários internos obsoletos após migração.

### Remoção de pastas e assets
- [ ] Remover pastas técnicas descontinuadas (runtime paths, planners, adapters legados).
- [ ] Remover demos/cenários antigos sem valor para vNext.
- [ ] Remover shaders/assets auxiliares sem referência.

### Remoção e atualização de documentação
- [ ] Remover docs antigas sem relevância para vNext.
- [ ] Atualizar docs sobreviventes para refletir somente a arquitetura nova.
- [ ] Remover páginas de comandos/tipos que não existem mais.

### Regras de segurança para remoção
- [ ] Antes de remover: confirmar ausência de uso com `rg` no workspace.
- [ ] Depois de remover: validar build/testes dos crates afetados.
- [ ] Confirmar que não restaram referências órfãs em código, docs e demos.

### Gates de conclusão de limpeza
- [ ] `rg` sem ocorrências de termos legados críticos na superfície pública.
- [ ] Workspace compila sem módulos/deps obsoletos.
- [ ] Docs e exemplos não mencionam conceitos removidos.
