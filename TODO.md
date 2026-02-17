# TODO — Completar Integração do egui no Vulfram

Base de referência (egui):
- https://docs.rs/egui/latest/egui/
- https://docs.rs/egui/latest/egui/containers/panel/struct.SidePanel.html
- https://docs.rs/egui/latest/egui/containers/struct.Window.html
- https://docs.rs/egui/latest/egui/containers/struct.CollapsingHeader.html
- https://docs.rs/egui/latest/egui/widgets/struct.TextEdit.html
- https://docs.rs/egui/latest/egui/widgets/struct.Slider.html
- https://docs.rs/egui/latest/egui/struct.Context.html
- https://docs.rs/egui/latest/egui/enum.Event.html
- https://docs.rs/egui/latest/egui/enum.OutputCommand.html
- https://docs.rs/epaint/latest/epaint/enum.Shape.html

## Fase 0 — Fundamentos e Contrato (P0)
- [x] Congelar escopo da UI API v1 (nós + props + eventos) alinhado ao egui atual.
- [x] Definir matriz de cobertura (recurso egui -> suporte no core -> comando/props).
- [x] Definir política de compatibilidade temporária (projeto experimental sem retrocompatibilidade).
- [x] Formalizar padrão de eventos de erro no pool de eventos ao host para falhas de UI/input/render.
- [x] Criar checklist de aceite por fase (funcional, performance, documentação).

## Fase 1 — Containers e Estrutura de Layout (P0)
- [x] Adicionar `UiNodeKind::Window` + props (title, open, movable, resizable, collapsible, anchored).
- [x] Adicionar `UiNodeKind::Panel` com variações: side/top-bottom/central.
- [x] Implementar `Split Pane` resizable estilo Blender (divisor arrastável por ponteiro, horizontal/vertical, min/max por lado, cursor de resize e persistência de proporção).
- [x] Adicionar `UiNodeKind::Area` (posição livre, drag opcional).
- [x] Adicionar `UiNodeKind::Frame` (margin, fill, stroke, rounding) para agrupar conteúdo com estilo.
- [x] Adicionar `UiNodeKind::ScrollArea` dedicado (em vez de flags no container).
- [x] Adicionar `UiNodeKind::Grid` dedicado (com striped/min_col_width etc.).
- [x] Adicionar containers de `Popup`, `Tooltip` e `Modal` (abertura/fechamento e ancoragem).
- [x] Adicionar `UiNodeKind::Resize` para regiões redimensionáveis fora de panel/window.
- [x] Adicionar `UiNodeKind::Scene` (pan/zoom) quando usado como container de visualização.
- [x] Garantir persistência de estado de layout (larguras de panel, open/closed de window/header).
- [x] Ajustar ordenação/z-index para windows e áreas sobrepostas no mesmo documento.
- [x] Validar clipping/hit-test de layout com rects dinâmicos e resize de janela.

## Fase 2 — Widgets Essenciais do egui (P0/P1)
- [x] Expandir tipos de widget:
- [x] `Label`/`RichText`/`Link`/`Hyperlink` com estados e eventos.
- [x] `Checkbox`, `Radio`, `SelectableLabel`, `Toggle`.
- [x] `Slider`, `DragValue`, `ProgressBar`.
- [x] `ComboBox`, `MenuButton`.
- [x] `CollapsingHeader`.
- [x] `ImageButton` e `Spinner`.
- [x] `TextEdit` multiline + password + char_limit.
- [x] Eventos por widget:
- [x] `click`, `doubleClick`, `pressed`, `released`.
- [x] `hoverEnter`, `hoverLeave`.
- [x] `changed` (contínuo) e `changeCommit`.
- [x] `focus`, `blur`, `submit`.
- [x] Mapear `enabled/disabled` para todos os novos widgets.
- [x] Suportar tooltips e context menus em nós interativos.

## Fase 3 — Painter e Paths (P1)
- [x] Adicionar nó/comando de desenho vetorial (`UiNodeKind::Canvas` ou `UiPaintOps`).
- [x] Cobrir primitivas de path:
- [x] line segment/polyline.
- [x] rect/rect_filled/rounded rect.
- [x] circle/circle_filled.
- [x] convex polygon fill + stroke.
- [x] bezier/quadratic path.
- [x] text paint com alinhamento, font, cor.
- [x] Expor stroke/fill styles (espessura, join, cap, alpha).
- [x] Integrar draw order do painter com z-index da árvore declarativa.
- [x] Suportar clipping por nó para painter.

## Fase 4 — Input Completo e Output de Plataforma (P0/P1)
- [x] Revisar mapeamento completo de `egui::Event`:
- [x] pointer moved/button/wheel/touch.
- [x] key/modifiers/text/IME.
- [x] zoom/pinch/pan/rotation gestures.
- [x] copy/cut/paste, `MouseMoved`, `WindowFocused`, `Screenshot` reply event.
- [x] `AccessKitActionRequest` (acessibilidade) com fallback quando indisponível.
- [x] Implementar processamento completo de `PlatformOutput`/`OutputCommand`:
- [x] cursor icon -> comandos de cursor da janela.
- [x] copy/cut/paste -> integração clipboard host.
- [x] copy image (`OutputCommand::CopyImage`) -> clipboard image do host.
- [x] open_url -> evento para host decidir.
- [x] request focus/attention -> encaminhar ao window subsystem.
- [x] Definir foco/captura por `windowId + realmId + documentId`.
- [x] Remover inconsistências entre posição real do ponteiro e UV convertida.
- [x] Garantir precisão do hit-test em targets com rect e cover-fit.
- [x] Garantir respeito ao z-index em roteamento de eventos.

## Fase 5 — Integração Realm/Target/Layer (P0)
- [ ] Revisar arquitetura de composição UI em targets:
- [ ] `window`, `texture`, `widget-realm-viewport`, `realm-plane`.
- [ ] Garantir contrato único de tamanho efetivo (host rect vs size real renderizado).
- [ ] Garantir cadeia de eventos cíclica entre realms/targets sem bloqueio.
- [ ] Consolidar roteamento pointer:
- [ ] host window -> target layer -> realm -> widget viewport -> realm 3D -> realm plane -> UI doc.
- [ ] Expor no trace hops detalhados com ids lógicos (realm/target/layer/surface/camera).
- [ ] Validar fallback de camera por layer (`camera_id` opcional).
- [ ] Validar environment por layer (`environment_id`) sem vazar entre binds.
- [ ] Garantir isolamento de render por camera/layer (sem acumulação indevida).

## Fase 6 — Multi-Viewport e Subjanelas do egui (P1/P2)
- [ ] Suportar `Context::show_viewport_*` quando houver backend capability.
- [ ] Mapear viewport egui <-> window/target do core.
- [ ] Definir modo fallback quando multi-viewport nativo não estiver disponível.
- [ ] Sincronizar ciclo de vida (create/update/close) entre egui viewport e window manager.
- [ ] Garantir input/foco corretos entre viewports múltiplos.
- [ ] Cobrir comandos principais de viewport: `Title`, `InnerSize`, `OuterPosition`, `Resizable`, `Decorations`, `Fullscreen`, `Minimized`, `Maximized`, `Cursor*`, `IME*`, `Focus`, `Screenshot`.

## Fase 7 — Tema, Estilo e Tipografia (P1)
- [ ] Expandir tema para cobrir `Style`/`Visuals` de forma abrangente.
- [ ] Expor tokens de spacing, rounding, stroke, widget states.
- [ ] Expor text styles por papel (heading/body/monospace/small/button).
- [ ] Suportar fontes custom por documento/tema.
- [ ] Suportar troca de tema sem rebuild completo de documento.

## Fase 8 — Performance e Memória (P0/P1)
- [ ] Substituir alocação por malha/frame no UI renderer por buffers persistentes (ring/suballoc).
- [ ] Reduzir clones em `CmdUiApplyOps` (journal + rollback local em vez de clone completo do doc).
- [ ] GC de `input_buffers` e `animations` por subtree removida.
- [ ] GC de texturas externas órfãs e handles de UI image.
- [ ] Métricas detalhadas de UI:
- [ ] layout, tessellation, upload de textura, draw, input routing.
- [ ] Sampling de trace para produção (`off/errors/basic/full`).
- [ ] Bench com cenários: 1k/5k nós, drag de splitter, múltiplos viewports embutidos.

## Fase 9 — Comandos e API Host (P0)
- [ ] Padronizar nomenclatura final (target/layer/realm widgets) sem aliases antigos.
- [ ] Adicionar comandos de introspecção UI:
- [ ] `CmdUiDocumentGetTree`.
- [ ] `CmdUiDocumentGetLayoutRects`.
- [ ] `CmdUiFocusSet/Get`.
- [ ] `CmdUiEventTraceSet` (nível e sampling).
- [ ] Revisar validadores de comando para erros explícitos e retorno padronizado.
- [ ] Garantir que erros diagnósticos sejam emitidos também como eventos no pool host.

## Fase 10 — Testes (P0)
- [ ] Testes unitários:
- [ ] ops de documento (add/remove/move/set/clear/versionamento).
- [ ] ordenação z-index.
- [ ] animações e fim de animação.
- [ ] conversão input host -> egui events.
- [ ] Testes de integração:
- [ ] widget realm viewport com pointer correto.
- [ ] realm plane com raycast + repasse de input.
- [ ] side panels resizable por ponteiro.
- [ ] multi-camera + multi-layer sem bleed.
- [ ] Testes visuais golden image para painter paths e widgets principais.
- [ ] Testes de stress (resize contínuo, criação/descarte em loop, 30min memória estável).

## Fase 11 — Demos de Validação (P1)
- [ ] Demo UI Widgets Showcase (todos widgets suportados).
- [ ] Demo Panels + Splitter + Dock-style básico.
- [ ] Demo Painter/Path.
- [ ] Demo Multi-Viewport (ou fallback documentado).
- [ ] Demo integração completa:
- [ ] UI principal com `WidgetRealmViewport`.
- [ ] Realm3D com `RealmPlane` interativo.
- [ ] Eventos ponta-a-ponta e trace habilitável.

## Fase 12 — Documentação Final (P0)
- [ ] Atualizar `docs/API.md` com todos comandos/propriedades novos.
- [ ] Atualizar `docs/cmds/*` (UI, target layer, environment por bind, event trace).
- [ ] Criar `docs/UI.md` técnico (arquitetura runtime/UI renderer/input/output).
- [ ] Criar `docs/UI-WIDGETS.md` (nós suportados + exemplos).
- [ ] Criar `docs/UI-EVENTS.md` (lista de eventos, ordem, semântica).
- [ ] Criar `docs/UI-PAINTER.md` (paths/primitives/clipping).
- [ ] Documentar limitações conhecidas e fallbacks de plataforma.
- [ ] Adicionar guia de migração para hosts após renomeações de API.

## Critérios de Conclusão
- [ ] Cobertura funcional >= 90% dos recursos-alvo do egui para uso em runtime.
- [ ] Sem crashes com variação de layout/input/resize/multi-target.
- [ ] Precisão de ponteiro consistente em todas as camadas realm/target/layer.
- [ ] Sem crescimento de memória não controlado em testes longos.
- [ ] Documentação consistente com o código atual.
