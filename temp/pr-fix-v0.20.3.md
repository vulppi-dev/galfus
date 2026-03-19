## Summary
- Corrige clamp de delta de ponteiro em captura (`locked` e `confined`) para não limitar rotação em cenários FPS.
- Adiciona validação prática no Demo 3: teclas `1/2/3` alternam `normal/locked/confined`, cubo rotaciona por delta, e UI mostra posição/delta.
- Atualiza documentação do cursor e do Demo 3 para refletir o novo comportamento.
- Atualiza versão do pacote para `0.20.3`.

## Linked Issues
- Closes #

## Change Type
- [ ] `changelog:breaking`
- [ ] `changelog:feature`
- [x] `changelog:fix`
- [ ] `changelog:performance`
- [x] `changelog:docs`
- [ ] `changelog:internal` (exclude from public changelog)

## Validation
- [x] `scripts/check.sh`
- [x] Relevant demo(s) manually tested
- [x] No unexpected regressions observed

## User-facing Changelog Note
Pointer delta is no longer clamped in `locked`/`confined`, enabling consistent FPS-style rotation; Demo 3 now includes runtime pointer diagnostics and mode switching via `1/2/3`.

## Risks / Rollback
- Main risks:
  - Alteração no caminho de input relativo pode mudar comportamento esperado de fluxos que dependiam implicitamente de clamp.
  - Demo 3 ganhou lógica de estado adicional para debug de ponteiro.
- Rollback plan:
  - Reverter o commit desta branch (`dev/fix-v0.20.3`) para restaurar o comportamento anterior.
