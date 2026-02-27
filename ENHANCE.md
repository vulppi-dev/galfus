# ENHANCE.md

## Objetivo
Este documento consolida a auditoria minuciosa dos arquivos `*.rs` e propõe correções detalhadas para reduzir consumo de CPU, memória e GPU, além de remover validações redundantes onde fizer sentido para o estágio experimental do projeto.

## Escopo da Auditoria
- Arquivos auditados: `203` arquivos `*.rs`.
- Tamanho auditado: `41.978` linhas.
- Foco: hotpaths de frame, sincronização de cena, input routing, UI, compose/post-process, bindings e áudio.
- Evidências coletadas por leitura de código + varreduras estáticas.

## Critérios de Priorização
- `P0`: alto impacto e risco de escalabilidade; atacar primeiro.
- `P1`: impacto médio e custo de implementação moderado.
- `P2`: ganho incremental ou ajuste de limpeza técnica.

## Sumário Executivo
- Existem clones integrais de mapas de recursos em caminhos de render por frame.
- Há rebuild completo de estruturas temporárias em loops de frame (input/UI/render).
- Compose/Post criam bind groups por câmera por frame, gerando overhead de CPU e driver.
- Existem cópias evitáveis em fronteiras de serialização e fila de comandos.
- Parte das validações de IDs pode ser simplificada no core, alinhando com a política atual de responsabilidade do host.

## Problema 01 (P0)
### Título
Clone integral de recursos globais na sincronização de cena.

### Evidência
- `src/core/render/mod.rs:956`
- `src/core/render/mod.rs:957`
- `src/core/render/mod.rs:959`

### Sintoma
A cada sincronização de cena ocorre clonagem integral de `textures`, `forward_atlas_entries` e `target_texture_binds`.

### Causa Raiz
Modelo de sync baseado em cópia total de mapas sem diff incremental.

### Impacto
- CPU: custo proporcional ao tamanho dos mapas.
- Memória: churn de alocação e cópia.
- Frame time: picos com aumento de recursos.

### Proposta de Correção
- Introduzir controle de sujeira (`dirty`) por coleção global.
- Aplicar sync incremental por chave adicionada/removida/alterada.
- Reaproveitar entradas estáveis sem `clone()`.

### Plano de Implementação
1. Adicionar versão/hash por coleção global.
2. Comparar versão anterior na `RenderState` antes de sincronizar.
3. Atualizar apenas delta por `insert/remove`.
4. Invalidar bind groups somente para materiais realmente afetados.

### Critérios de Aceite
- Sem clone integral em frame sem mudanças.
- Queda de CPU no trecho de sync sob carga de recursos.

### Risco
Médio: exige cuidado com consistência de invalidadores de cache.

---

## Problema 02 (P0)
### Título
Rebuild total de `external_textures` para cada janela a cada frame.

### Evidência
- `src/core/render/mod.rs:1362`

### Sintoma
`external_textures.clear()` seguido de reinserção completa em loop de estados de render.

### Causa Raiz
Ausência de estratégia de atualização incremental para vínculos externos.

### Impacto
- CPU: custo repetitivo em todos os frames.
- Memória: realocação/rehash recorrente.

### Proposta de Correção
- Substituir limpeza total por atualização dif incremental.
- Usar cache de chave `{texture_id, target_id, surface_ptr}` para detectar mudanças.
- Preservar entradas estáveis sem tocar no `HashMap`.

### Plano de Implementação
1. Manter índice anterior por janela.
2. Atualizar apenas entradas mutadas.
3. Remover apenas entradas órfãs no final.

### Critérios de Aceite
- Sem `clear()` global por frame.
- Estabilidade de frame time com muitos bindings externos.

### Risco
Baixo.

---

## Problema 03 (P0)
### Título
Compose cria bind group por câmera por frame e limpa alvo sempre.

### Evidência
- `src/core/render/passes/compose/mod.rs:130`
- `src/core/render/passes/compose/mod.rs:149`
- `src/core/render/passes/compose/mod.rs:214`

### Sintoma
Bind groups são recriados em loop de câmeras, além de `LoadOp::Clear` fixo.

### Causa Raiz
Ausência de cache de bind groups de compose e política de load/store rígida.

### Impacto
- CPU/driver: criação recorrente de objetos GPU-side.
- GPU: clear redundante em cenários onde já houve clear anterior adequado.

### Proposta de Correção
- Introduzir cache de bind group por assinatura de views (`target/outline/ssao/bloom`).
- Avaliar troca para `LoadOp::Load` quando semanticamente seguro.
- Separar atualização de uniformes por câmera sem recriar bind group.

### Plano de Implementação
1. Criar `ComposeBindCache` em `RenderState`.
2. Chavear cache por ponteiro/ID de textura + formato.
3. Implementar invalidação ao recrear targets.
4. Medir diferença de CPU no pass.

### Critérios de Aceite
- Redução significativa de criação de bind group por frame.
- Render visual idêntico aos testes de referência.

### Risco
Médio: invalidadores de cache precisam ser precisos.

---

## Problema 04 (P0)
### Título
Post-process repete padrão custoso (clone/config + bind group por câmera + clear).

### Evidência
- `src/core/render/passes/post/mod.rs:138`
- `src/core/render/passes/post/mod.rs:139`
- `src/core/render/passes/post/mod.rs:155`
- `src/core/render/passes/post/mod.rs:237`
- `src/core/render/passes/post/mod.rs:253`

### Sintoma
Clones de configuração e criação de bind group em loop de câmeras.

### Causa Raiz
Estrutura similar ao compose sem cache de bindings e com cópias de config.

### Impacto
- CPU: alto em múltiplas câmeras.
- GPU: clear redundante em alguns encadeamentos de pass.

### Proposta de Correção
- Mesma abordagem do compose: cache de bind groups e acesso por referência às configs.
- Evitar `clone()` de `camera_environment_overrides` e `default_post`.
- Validar necessidade real de `Clear(Black)` por caso.

### Plano de Implementação
1. Eliminar clones de mapa/config.
2. Criar cache de bind group para post.
3. Revisar política de clear/load com testes visuais.

### Critérios de Aceite
- Redução de alocação no pass post.
- Sem regressão visual em outlines/ssao/bloom.

### Risco
Médio.

---

## Problema 05 (P0)
### Título
Input routing reconstrói vários mapas e ordenações por tick.

### Evidência
- `src/core/input/routing.rs:24`
- `src/core/input/routing.rs:31`
- `src/core/input/routing.rs:38`
- `src/core/input/routing.rs:50`
- `src/core/input/routing.rs:56`
- `src/core/input/routing.rs:63`
- `src/core/input/routing.rs:91`

### Sintoma
Múltiplos `HashMap`/`Vec` temporários e sort em toda execução do roteamento.

### Causa Raiz
Pipeline de roteamento sem cache estrutural persistente entre frames.

### Impacto
- CPU: custo fixo por frame, piora com crescimento de conectores/layers/targets.
- Latência de input: pior em cenas densas.

### Proposta de Correção
- Criar cache incremental de roteamento com invalidação por mudanças de graph/targets/layers/connectors.
- Manter conectores já ordenados por realm enquanto a topologia não muda.

### Plano de Implementação
1. Adicionar `routing_cache` em `UniversalState` ou `InputState`.
2. Invalidation hooks nos comandos que mudam estrutura.
3. Rebuild total apenas quando necessário.

### Critérios de Aceite
- Queda do custo de `route_pointer_events` em frames sem mutação estrutural.

### Risco
Médio.

---

## Problema 06 (P0)
### Título
Cópia evitável de bytes em `vulfram_send_queue`.

### Evidência
- `src/core/queue.rs:17`
- `src/core/queue.rs:19`

### Sintoma
`from_raw_parts(...).to_vec()` antes de desserializar MessagePack.

### Causa Raiz
Uso de buffer proprietário onde bastaria slice imutável.

### Impacto
- CPU/memória: cópia extra por lote de comandos.

### Proposta de Correção
- Desserializar diretamente do slice original.
- Manter `unsafe` confinado com validações mínimas de ponteiro e tamanho.

### Plano de Implementação
1. Trocar decode para `from_slice(raw_slice)`.
2. Remover alocação intermediária.
3. Cobrir com teste de regressão para payloads grandes.

### Critérios de Aceite
- Sem alocação temporária para decode de input queue.

### Risco
Baixo.

---

## Problema 07 (P1)
### Título
Pass UI limpa e reconstrói texturas externas por frame.

### Evidência
- `src/core/render/passes/ui/mod.rs:539`
- `src/core/render/passes/ui/mod.rs:540`
- `src/core/render/passes/ui/mod.rs:562`
- `src/core/render/passes/ui/mod.rs:593`

### Sintoma
`ui_state.external_textures.clear()` e rebuild de `target_surfaces`/`inputs` em toda chamada.

### Causa Raiz
Sem cache por realm/window para mapeamento de targets externos.

### Impacto
CPU e churn de `HashMap`/`Vec` em UI pesada.

### Proposta de Correção
- Cachear `external_inputs` por realm com invalidação em mudanças de auto-links/surfaces/targets.
- Atualizar apenas delta de texturas externas.

### Critérios de Aceite
Redução de custo na etapa de setup da pass UI.

### Risco
Baixo.

---

## Problema 08 (P1)
### Título
Clones de tesselação na pass UI.

### Evidência
- `src/core/render/passes/ui/mod.rs:191`
- `src/core/render/passes/ui/mod.rs:201`

### Sintoma
`cache.clipped.clone()` e novo clone ao armazenar cache.

### Causa Raiz
Estratégia de cache baseada em ownership por cópia.

### Impacto
- Memória/CPU: cópias de primitivas potencialmente grandes.

### Proposta de Correção
- Mudar cache para `Arc<[ClippedPrimitive]>` (ou estrutura equivalente).
- Reutilizar fatias sem copiar quando hash e `pixels_per_point` não mudarem.

### Critérios de Aceite
Queda de alocação em frames de UI estável.

### Risco
Médio.

---

## Problema 09 (P1)
### Título
`process_ui_input` aloca vetores transitórios por tick.

### Evidência
- `src/core/ui/input.rs:15`
- `src/core/ui/input.rs:16`
- `src/core/ui/input.rs:17`
- `src/core/ui/input.rs:18`

### Sintoma
Quatro vetores são criados por chamada.

### Causa Raiz
Falta de buffers reutilizáveis no estado de UI input.

### Impacto
Ganho pequeno por frame, mas contínuo.

### Proposta de Correção
- Introduzir scratch buffers persistentes em `UiState`.
- Limpar com `clear()` mantendo capacidade.

### Critérios de Aceite
Redução de alocações por frame no processamento de input.

### Risco
Baixo.

---

## Problema 10 (P1)
### Título
Forward pass materializa mapas auxiliares por frame.

### Evidência
- `src/core/render/passes/forward/mod.rs:93`
- `src/core/render/passes/forward/mod.rs:98`
- `src/core/render/passes/forward/mod.rs:104`
- `src/core/render/passes/forward/mod.rs:105`

### Sintoma
`HashMap` temporários para clear color, skybox e sample count.

### Causa Raiz
Pré-processamento por coleção em vez de acesso direto por câmera.

### Impacto
CPU moderada em cenários multicâmera.

### Proposta de Correção
- Calcular valores on-demand por câmera (ou cache fixo por frame sem `HashMap`).
- Usar `Vec<(camera_id, sample_count)>` ordenado ao invés de mapas quando possível.

### Critérios de Aceite
Menor overhead no setup da pass forward.

### Risco
Baixo.

---

## Problema 11 (P1)
### Título
`prepare_lights` aloca/sorta estruturas temporárias por frame.

### Evidência
- `src/core/render/state/prepare/lights.rs:9`
- `src/core/render/state/prepare/lights.rs:11`
- `src/core/render/state/prepare/lights.rs:39`

### Sintoma
Criação de `lights`, `sorted_lights` e `frustums` em toda preparação.

### Causa Raiz
Sem scratch buffers persistentes.

### Impacto
CPU + alocação incremental em cenas com muitas luzes/câmeras.

### Proposta de Correção
- Guardar buffers temporários no `RenderState`.
- Reusar capacidade e reduzir sort quando ordem de IDs não mudou.

### Critérios de Aceite
Menor churn de memória no prepare de luzes.

### Risco
Baixo.

---

## Problema 12 (P1)
### Título
`realm_graph` faz sort para hash e clear de cache completo.

### Evidência
- `src/core/render/realm_graph.rs:64`
- `src/core/render/realm_graph.rs:65`
- `src/core/render/realm_graph.rs:83`

### Sintoma
Ordenação de presents e `cache.clear()` em atualização.

### Causa Raiz
Hash global e invalidação ampla em vez de incremental.

### Impacto
CPU adicional em mudanças frequentes de janela/surface.

### Proposta de Correção
- Hash incremental por surface/window.
- Atualizar apenas entradas alteradas do cache.

### Critérios de Aceite
Menor custo quando poucas presents mudam.

### Risco
Médio.

---

## Problema 13 (P1)
### Título
Muitos clones na renderização declarativa de UI.

### Evidência
- `src/core/ui/render.rs:86`
- `src/core/ui/render.rs:125`
- `src/core/ui/render.rs:128`
- `src/core/ui/render.rs:2166`

### Sintoma
Clone de documento e de listas (`to_vec`) em caminhos de render.

### Causa Raiz
Estratégia de ownership conservadora para contornar empréstimos mutáveis.

### Impacto
CPU/memória em árvores UI grandes.

### Proposta de Correção
- Refatorar render para separar leitura imutável e escrita de estado em duas fases.
- Evitar `doc.clone()` integral.
- Trocar `to_vec()` por iteração sobre slices/caches precomputados.

### Critérios de Aceite
Redução de alocação por frame em cenas UI extensas.

### Risco
Alto: refactor estrutural com atenção a borrowing/eventos.

---

## Problema 14 (P2)
### Título
`prepare_materials` monta `Vec<BindGroupEntry>` dinamicamente.

### Evidência
- `src/core/render/state/prepare/materials.rs:95`
- `src/core/render/state/prepare/materials.rs:242`

### Sintoma
Montagem repetitiva de vetor de entries quando bind group precisa ser recriado.

### Causa Raiz
Construção dinâmica sem reservação explícita ou estrutura fixa.

### Impacto
Ganho incremental (não crítico) em cenas com mutação frequente de material.

### Proposta de Correção
- Reservar capacidade exata (`with_capacity`) e/ou usar array estático quando viável.
- Reavaliar frequência de invalidação de bind group.

### Critérios de Aceite
Menos alocação durante rebuild de material.

### Risco
Baixo.

---

## Problema 15 (P2)
### Título
Cópia no getter `buffer()` da exportação WASM.

### Evidência
- `src/lib.rs:20`

### Sintoma
Getter retorna `Vec<u8>` por cópia (`clone`).

### Causa Raiz
Interface exposta por valor sem consumo único.

### Impacto
Cópia extra na fronteira host/core para payloads grandes.

### Proposta de Correção
- Avaliar API com consumo (`take_buffer`) ou representação que evite clone.
- Manter compatibilidade com o binding alvo.

### Critérios de Aceite
Sem cópia extra ao recuperar buffers serializados.

### Risco
Médio: depende de contrato da API exposta.

---

## Validações Redundantes/Inúteis (Análise)
### Contexto
A diretriz atual define que o host garante unicidade e validade de IDs lógicos; o core pode assumir validade.

### Pontos Identificados
- Verificações de existência de realm em comandos de áudio:
  - `src/core/audio/cmd.rs:388`
  - `src/core/audio/cmd.rs:633`
  - `src/core/audio/cmd.rs:715`

### Proposta
- Reduzir validações de existência em hotpaths de comando quando o contrato host/core já cobre o cenário.
- Manter validações apenas onde o erro afeta segurança interna, corrupção de estado ou geração de eventos críticos.

### Observação Importante
Mesmo simplificando validações, manter emissão consistente de `SystemEvent::Error` para falhas diagnosticáveis que ainda possam ocorrer em runtime.

## Plano de Execução Recomendado
### Fase 1 (alto ROI)
1. Problema 06 (`queue` cópia evitável).
2. Problema 01 (sync incremental de recursos globais).
3. Problema 02 (diff em `external_textures`).
4. Problema 03 (cache compose bind groups).
5. Problema 04 (cache post bind groups e remoção de clones).
6. Problema 05 (cache incremental de routing).

### Fase 2 (médio ROI)
1. Problema 07 (cache de inputs externos UI).
2. Problema 08 (cache tesselação sem clone).
3. Problema 09 (scratch buffers UI input).
4. Problema 10 (simplificação setup forward).
5. Problema 11 (scratch buffers de lights).
6. Problema 12 (hash/cache incremental realm graph).

### Fase 3 (limpeza e estrutural)
1. Problema 13 (refactor UI render sem clones amplos).
2. Problema 14 (alocação de entries de materiais).
3. Problema 15 (otimização da fronteira WASM).
4. Revisão final de validações redundantes no áudio.

## Métricas de Sucesso
- CPU frame time médio e p95 antes/depois por cenário de demo.
- Contagem de alocações por frame nas etapas de render/input/ui.
- Tempo de execução dos passes: `forward`, `post`, `compose`, `ui`.
- Estabilidade visual: comparação de frames de referência.
- Latência de input em cenários com muitos conectores.

## Estratégia de Benchmark
- Cenário A: 1 janela, 1 câmera, UI leve.
- Cenário B: 1 janela, múltiplas câmeras, pós-processamento ativo.
- Cenário C: múltiplas realms/conectores com overlap de input.
- Cenário D: UI densa com imagens e atualizações de textura.
- Cenário E: carga alta de recursos globais (texturas/materials).

## Política de Segurança e Consistência
- Evitar alterações destrutivas de API sem mapeamento de migração.
- Garantir rollback local em caso de falha de comando.
- Emitir eventos de erro quando aplicável.
- Preservar comportamento visual durante otimização de passes.

## Dependências Técnicas
- Estruturas de cache em `RenderState` e/ou `UniversalState`.
- Invalidation hooks consistentes em comandos que alteram topologia.
- Telemetria mínima por subsistema para medir ganhos reais.

## Riscos Gerais
- Cache inválido causar artefato visual intermitente.
- Otimização prematura em áreas de baixo impacto.
- Refactor de UI aumentar complexidade de borrowing.

## Mitigação de Riscos
- Implementar por fases com feature flags internas.
- Adicionar testes de regressão focados em invalidação de cache.
- Validar visualmente cada fase com demos e snapshots.

## Decisões Arquiteturais Recomendadas
- Preferir sincronização incremental em todos os hotpaths de frame.
- Evitar clones integrais de mapas em loop de render.
- Usar buffers scratch persistentes para estruturas temporárias.
- Isolar otimizações de API externa com contrato claro para host.

## Checklist de Implementação
- [ ] Remover cópia intermediária de queue decode.
- [ ] Introduzir versão/hash de recursos globais para sync incremental.
- [ ] Trocar rebuild de `external_textures` por diff incremental.
- [ ] Cachear bind groups de compose.
- [ ] Cachear bind groups de post.
- [ ] Reduzir clones de config em compose/post.
- [ ] Criar cache incremental de input routing.
- [ ] Cachear mapeamento de external textures na pass UI.
- [ ] Evitar clones de tesselação na pass UI.
- [ ] Adicionar scratch buffers no `process_ui_input`.
- [ ] Reduzir estruturas temporárias no forward setup.
- [ ] Adicionar scratch buffers de lights.
- [ ] Tornar `realm_graph` incremental em hash/cache.
- [ ] Planejar refactor de `ui/render.rs` para remover clones amplos.
- [ ] Revisar validações redundantes no áudio conforme contrato host/core.

## Checklist de Validação
- [ ] `scripts/check.sh`.
- [ ] Testes de regressão visual dos demos.
- [ ] Medição de profiling antes/depois por fase.
- [ ] Verificação de estabilidade de input routing.
- [ ] Verificação de integridade dos eventos de erro.

## Estado Atual
Documento criado para orientar implementação de melhoria contínua em fases, com foco inicial nos itens de maior retorno em CPU/memória/GPU.
