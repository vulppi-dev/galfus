# EVENTS Roadmap and Implementation Guide

Documento de referência para implementação das melhorias no sistema de eventos.
Foco: consistência cross-platform, previsibilidade de payload, baixo risco de regressão, eficácia e performance.

## 1. Objetivo
Evoluir o pipeline de eventos (keyboard, pointer e gamepad) com quatro entregas:
1. Unificação de keycodes entre desktop e browser.
2. Correção de defasagem de `OnAxis` no gamepad.
3. Eventos dedicados de teclado no browser (modifiers e IME).
4. Relatividade de ponteiro por target, mantendo coordenada global por janela.

Também definir um subsistema de **listeners por target** com lifecycle explícito e dispose em cascata.

## 2. Princípios de implementação
- **Fonte única de verdade** para tabelas e contratos de payload.
- **Sem ambiguidade de coordenadas**: global e relativo coexistem com nomes explícitos.
- **Custo por frame controlado**: evitar alocações desnecessárias e buscas lineares em hot paths.
- **Fail-safe**: quando um dado relativo não puder ser resolvido, evento segue válido com fallback.
- **Dispose determinístico**: remover listeners automaticamente quando target for removido.

## 3. Escopo técnico por entrega

### 3.1 Unificação de keycodes

#### Problema
Browser e desktop usam mapeamentos numéricos diferentes em parte das teclas, causando divergência em atalhos e no consumo da UI.

#### Estratégia
- Criar tabela canônica única (ex.: `core/input/keycodes.rs`).
- Converter desktop e browser para usar os mesmos IDs lógicos.
- Centralizar helper de mapeamento para UI/demos com base nessa mesma tabela.

#### Implementação sugerida
1. Extrair constantes de keycode para módulo compartilhado.
2. Refatorar `convert_key_code` (desktop) e `map_key_code` (browser) para a tabela canônica.
3. Atualizar consumidores que dependem de valores fixos (`Escape`, `Ctrl+W`, etc.).
4. Adicionar testes de paridade desktop/browser para subset crítico.

#### Critérios de aceite
- Mesmo input físico gera mesmo `key_code` em desktop e browser.
- Atalhos de fechamento e navegação funcionam de forma idêntica.

#### Risco e mitigação
- Risco: quebra de consumidor externo que assumiu códigos antigos.
- Mitigação: registrar breaking change no changelog interno e mapear atalhos sensíveis em testes.

---

### 3.2 Correção de defasagem do gamepad (`OnAxis`)

#### Problema
Fluxo atual pode publicar valor anterior do eixo por ordem de leitura/escrita no cache.

#### Estratégia
- Atualizar cache primeiro e emitir o valor ajustado atual no mesmo frame.
- Preservar dead-zone e threshold para não aumentar ruído.

#### Implementação sugerida
1. No caminho de `AxisChanged`, calcular `adjusted_value` a partir do input atual.
2. Persistir no cache e publicar esse mesmo valor no evento.
3. Revisar implementação web para manter semântica equivalente.

#### Critérios de aceite
- `OnAxis.value` sempre representa estado corrente do frame.
- Sem regressão de volume de eventos quando controle está parado.

#### Risco e mitigação
- Risco: aumentar frequência de eventos em controles ruidosos.
- Mitigação: manter `GAMEPAD_AXIS_CHANGE_THRESHOLD` e dead-zone existentes.

---

### 3.3 Eventos dedicados de teclado no browser

#### Problema
Browser concentra informação em `OnInput`, sem paridade completa com desktop para modificadores e IME.

#### Estratégia
- Emitir `OnModifiersChange` quando houver transição real de modificadores.
- Emitir eventos de IME dedicados (enable/preedit/commit/disable) a partir de composition/input.

#### Implementação sugerida
1. Adicionar estado local de modifiers por janela (cache simples no core/browser).
2. Comparar estado anterior/atual e emitir `OnModifiersChange` somente em mudança.
3. Registrar listeners DOM de composition:
   - `compositionstart` -> `OnImeEnable`
   - `compositionupdate` -> `OnImePreedit`
   - `compositionend` -> `OnImeCommit` + `OnImeDisable`
4. Garantir coexistência com `keydown/keyup` sem duplicar texto indevidamente.

#### Critérios de aceite
- Browser entrega o mesmo conjunto semântico de eventos de teclado do desktop.
- Fluxo IME chega ao core de forma previsível (preedit + commit).

#### Risco e mitigação
- Risco: comportamento diferente entre navegadores.
- Mitigação: fallback conservador e testes em pelo menos 2 engines de browser.

---

### 3.4 Relatividade de ponteiro por target

#### Problema
Hoje o payload não explicita de forma padronizada a posição relativa ao target resolvido.

#### Objetivo de payload
Manter compatibilidade sem perder clareza:
- `position_global`: sempre relativo à janela (semântico global).
- `position_target`: opcional, relativo ao target resolvido.
- `target_space`: metadados opcionais para identificação do espaço relativo (target/connector/source realm).

#### Estratégia
- Calcular `position_target` durante o roteamento (`route_pointer_events`) quando houver target válido.
- Preservar evento mesmo sem target (apenas global).

#### Implementação sugerida
1. Evoluir `PointerEvent` com campos opcionais para posição relativa.
2. No roteamento, quando existir `uv + target/surface size`, projetar para pixels do target.
3. Garantir clamp e validação de faixa para evitar valores inválidos.
4. Revisar consumo em UI para usar relativo quando aplicável.

#### Critérios de aceite
- Todo evento continua com posição global válida.
- Quando target existe, payload inclui posição relativa coerente.
- Host consegue escolher explicitamente entre global e target-relative.

#### Risco e mitigação
- Risco: inflação de payload por evento.
- Mitigação: novos campos opcionais e preenchimento apenas quando resolvido.

## 4. Novo subsistema: listeners por target

### 4.1 Objetivo
Permitir registro de listeners para eventos associados a um target lógico específico, com controle explícito de lifecycle e limpeza automática.

### 4.2 Modelo de ownership
- `listener_id`: ID lógico fornecido pelo host.
- `target_id`: target lógico ao qual o listener pertence.
- Core gerencia indexação e dispatch.
- Host garante unicidade/validade dos IDs lógicos.

### 4.3 Novos comandos propostos

#### `cmd-input-target-listener-upsert`
Cria ou atualiza listener vinculado a target.

Payload sugerido:
- `windowId: u32`
- `listenerId: u64`
- `targetId: u64`
- `enabled: bool` (default true)
- `events: String[]` (ex.: `pointer-move`, `pointer-button`, `pointer-scroll`, `keyboard-input`, `gamepad-axis`)
- `scope: "target" | "target-and-descendants"` (default `target`)
- `throttleMs: u32` (opcional, default 0)
- `samplePercent: u8` (opcional, default 100)

Resposta sugerida:
- `success: bool`
- `message: String`

#### `cmd-input-target-listener-dispose`
Remove listener por ID.

Payload sugerido:
- `windowId: u32`
- `listenerId: u64`

Resposta sugerida:
- `success: bool`
- `message: String`

#### `cmd-input-target-listener-list` (opcional para debug)
Lista listeners vinculados a uma janela/target.

### 4.4 Eventos emitidos para host
Quando listener casar com evento:
- `SystemEvent::InputTargetListenerEvent` (ou evento dedicado `EngineEvent::InputTargetListener`)
- Campos mínimos:
  - `listenerId`
  - `targetId`
  - `eventType`
  - `timestamp`
  - `payload` (evento original enxuto + posições global/relativa quando houver)

### 4.5 Regras de dispose e cascata

#### Regra principal
- `target dispose => dispose automático de todos os listeners vinculados ao target`.

#### Regras complementares
- `window dispose/close => dispose de listeners dos targets daquela janela`.
- `listener dispose explícito` sempre idempotente.
- Durante cascata, não gerar erro fatal para listener já inexistente; registrar resultado como no-op.

#### Evento de auditoria (opcional)
Emitir evento de sistema de limpeza:
- `SystemEvent::InputTargetListenerDisposed { listenerId, reason }`
- `reason`: `explicit`, `target_dispose`, `window_dispose`, `shutdown`

## 5. Plano de rollout com baixo risco
1. **Fase A**: keycode + gamepad axis (alteração de lógica sem aumentar payload).
2. **Fase B**: eventos dedicados browser (modifiers/IME).
3. **Fase C**: campos relativos no pointer (payload opcional).
4. **Fase D**: comandos de listener por target + cascata de dispose.

Cada fase deve ser mergeável de forma independente.

## 6. Performance e eficácia
- Usar índices O(1):
  - `listeners_by_target: HashMap<TargetId, Vec<ListenerId>>`
  - `listener_state: HashMap<ListenerId, ListenerConfig>`
- Evitar alocação por evento em hot path:
  - reutilizar vetores temporários (scratch buffers).
- Throttle/sampling por listener para cenários de alto volume.
- Aplicar filtros antes de serializar payload (reduz custo de MessagePack).

## 7. Estratégia de testes

### 7.1 Testes funcionais
- Paridade de keycodes desktop/browser para conjunto crítico.
- Gamepad axis: validar ausência de defasagem.
- Browser IME: preedit/commit/disable em sequência válida.
- Pointer global + relativo ao target com e sem target resolvido.
- Dispose em cascata: target dispose remove listeners associados.

### 7.2 Testes de regressão
- Atalhos atuais (Escape, Ctrl+W).
- UI input routing sem regressão.
- Volume de eventos sob movimento contínuo (mouse/stick).

### 7.3 Testes de performance
- Benchmark curto de throughput de eventos por frame.
- Comparar serialização antes/depois em cenários de carga.

## 8. Checklist de execução
- [x] Criar tabela canônica de keycodes.
- [x] Unificar mapeamento desktop/browser.
- [x] Corrigir fluxo de `OnAxis` para valor corrente.
- [x] Implementar `OnModifiersChange` no browser.
- [x] Implementar eventos IME dedicados no browser.
- [x] Adicionar `position_target` opcional nos eventos de pointer.
- [x] Definir e implementar `cmd-input-target-listener-upsert`.
- [x] Definir e implementar `cmd-input-target-listener-dispose`.
- [x] Implementar dispose em cascata no `target dispose`.
- [x] Atualizar docs de API/comandos/eventos após cada fase.

## 9. Decisões pendentes
- Nome final do evento de retorno dos listeners (`SystemEvent` vs `EngineEvent` dedicado).
- Granularidade final de `events` no listener (lista fechada vs enum expansível).
- Estratégia de compatibilidade para hosts que consumirem payload antigo de pointer.

## 10. Backlog executável (arquivos e ordem)

### Task 1 - Base canônica de keycodes (iniciar por esta)
Objetivo: remover divergência de IDs e preparar base única para desktop/browser/UI/demos.

Arquivos alvo:
- `src/core/input/keycodes.rs` (novo): constantes canônicas + mapper para `KeyboardEvent.code` do browser.
- `src/core/input/mod.rs`: expor módulo de keycodes.
- `src/core/platforms/browser/input.rs`: substituir tabela local por mapper canônico.
- `src/core/ui/input.rs`: usar constantes para atalhos críticos (`Escape`).
- `src/demo/loop_utils.rs` e `src/demo/scenarios.rs`: trocar literais por constantes canônicas.

Entrega mínima:
- Browser passa a emitir os mesmos IDs lógicos de desktop para teclas cobertas no mapper.
- Consumidores deixam de depender de números mágicos para atalhos principais.

### Task 2 - Corrigir defasagem de `OnAxis` no gamepad
Objetivo: garantir publicação de valor corrente no mesmo frame.

Arquivos alvo:
- `src/core/gamepad/mod.rs`
- `src/core/gamepad/cache.rs` (se necessário para helper explícito de valor ajustado)
- testes em `src/core/input/tests_phase10.rs` ou módulo dedicado de gamepad.

Entrega mínima:
- `OnAxis.value` sem atraso de 1 evento.

### Task 3 - Eventos dedicados de teclado no browser
Objetivo: paridade semântica com desktop para modifiers e IME.

Arquivos alvo:
- `src/core/platforms/browser/input.rs`
- `src/core/input/cache.rs` (se optar por cache global de modifiers)
- `src/core/input/events/keyboard.rs` (apenas se for necessário ampliar payload)

Entrega mínima:
- emissão de `OnModifiersChange`.
- emissão de `OnImeEnable`, `OnImePreedit`, `OnImeCommit`, `OnImeDisable`.

### Task 4 - Posição relativa por target no pointer
Objetivo: manter global e incluir relativo quando target for resolvido.

Arquivos alvo:
- `src/core/input/events/pointer.rs` (payload)
- `src/core/input/routing.rs` (cálculo do relativo)
- `src/core/ui/input.rs` (consumo opcional do relativo)
- documentação de eventos.

Entrega mínima:
- payload com `position_global` + `position_target` opcional.

### Task 5 - Listener por target (upsert/dispose/list) + cascata
Objetivo: observabilidade seletiva por target com lifecycle robusto.

Arquivos alvo:
- `src/core/cmd.rs` (novos comandos/respostas/eventos)
- `src/core/input/*` (state/indexação/dispatch)
- `src/core/target/cmd.rs` e fluxo de dispose para cascata automática
- docs de comandos em `docs/cmds/*`.

Entrega mínima:
- `cmd-input-target-listener-upsert`
- `cmd-input-target-listener-dispose`
- dispose automático ao remover target.
