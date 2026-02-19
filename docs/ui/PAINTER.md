# RealmUI Painter

`UiNodeKind::Canvas` usa `UiPaintOp[]` para desenho vetorial declarativo.

## Primitivas

- `line-segment`
- `polyline`
- `rect`
- `rect-filled`
- `circle`
- `circle-filled`
- `convex-polygon`
- `quadratic-bezier`
- `cubic-bezier`
- `text`

## Estilos

- Stroke via `UiPaintStroke`:
  - `width`
  - `color`
  - `join?`
  - `cap?`
- Fill via `UiColor`.
- Texto:
  - `position`
  - `text`
  - `size?`
  - `color`
  - `align?`

## Clipping e Ordem

- `Canvas.clip` controla clipping no nó.
- Ordem respeita a árvore UI + `zIndex`.
- `Canvas` participa do mesmo fluxo de hit/layout do documento.
