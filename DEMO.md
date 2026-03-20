# DEMO.md

## Objetivo

Demos no Vulfram validam, de forma incremental, os principais aspectos da engine (core, render, janela, input, UI, recursos, áudio e eventos), sem compromisso de retrocompatibilidade nesta fase experimental.

## Padrões Globais (obrigatórios)

1. Toda janela deve fechar por:
   - evento de fechamento da janela
   - atalho `Ctrl+W`
2. Todo demo deve ter UI com FPS em tempo real visível em tela.
3. Todo demo 3D deve incluir um `plane` para simular chão e manter sombras habilitadas.

## Estrutura Base dos Novos Demos

1. `setup`:
   - criar janela/realm
   - preparar cena/estado mínimo
   - registrar HUD de FPS
2. `runtime`:
   - loop principal
   - atualização de estado
   - atualização de FPS
3. `input/events`:
   - tratar close event
   - tratar `Ctrl+W`
4. `teardown`:
   - encerrar sessão de forma limpa
   - descartar recursos temporários do demo

## Demos Executáveis

1. `Demo 1`
   - baseline de lifecycle, janela, realm, target, câmera e cena básica.
2. `Demo 2`
   - fluxo de janela orientado por UI: medições (`window-measurement`) e botões para mudança de estado (`window-state`/`window-cursor`) em um único realm UI.
3. `Demo 3`
   - recursos de cena e render (geometria, materiais, luzes, texturas, ambiente, sombras e pós).
   - validação de ponteiro para FPS: teclas `1/2/3` alternam `normal/locked/confined`, cubo rotaciona por delta do mouse em `locked` e `confined`, UI exibe posição e delta do ponteiro.
   - inclui suíte de gizmos para teste visual: `line`, `aabb`, `polyline` aberta/fechada e curva (Bezier amostrada), com espessura em pixels.
4. `Demo 4`
   - UI runtime completa, introspecção, viewport/raycast e integração 3D (sem gizmos nesta fase).
5. `Demo 5`
   - integração multi-sistema de runtime (multi-janela, áudio, input e diagnósticos).
6. `Demo 6`
   - contrato global (render graph, ABI/filas e cenário integrado de regressão manual).
7. `Demo 7`
   - validação funcional de listeners de ponteiro com split de tela: realm 3D à esquerda, UI à direita e `WidgetRealmViewport` interno para target aninhado.
