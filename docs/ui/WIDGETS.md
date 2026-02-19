# RealmUI Widgets e Nós

`UiNodeKind` suportados atualmente:

- Estrutura/layout:
  - `container`, `window`, `panel`, `split-pane`, `area`, `frame`
  - `scroll-area`, `grid`, `popup`, `tooltip`, `modal`, `resize`, `scene`
- Desenho:
  - `canvas`
- Texto/ação:
  - `text`, `rich-text`, `link`, `hyperlink`, `button`
- Seleção/estado:
  - `checkbox`, `radio`, `selectable-label`, `toggle`
- Valor:
  - `slider`, `drag-value`, `progress-bar`, `combo-box`, `menu-button`
  - `collapsing-header`, `spinner`
- Entrada:
  - `text-edit`, `input`
- Imagem/viewport:
  - `image`, `image-button`, `widget-realm-viewport`
- Auxiliares:
  - `separator`, `spacer`

## Propriedades Comuns

`UiNode` inclui:

- `display`: remove do layout/hit-test quando `false`.
- `visible`: invisível e não interativo quando `false`.
- `opacity`: multiplicador de opacidade (`0..1`).
- `zIndex`: ordenação no documento.
- `tooltip` e `contextMenu`.
- `anim`: animações declarativas (`opacity`, `translateY`).

## Tamanhos

- `UiLength`: `auto`, `fill`, `px`.
- `UiSize`: `{ width, height }`.

## Node Props

A referência completa de payload por nó está no tipo:

- `src/core/ui/types.rs`:
  - `UiNodeProps`
  - `UiLayout`, `UiPadding`, `UiColor`, `UiStroke`
  - `UiPaintOp` (para `canvas`)
