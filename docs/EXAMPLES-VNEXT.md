# vNext Canonical Examples

## 1. Same realm in multiple targets

Goal: render the same realm to a window and to a texture in the same frame.

```text
Target Window(1920x1080)
  Layer: Realm A, full

Target Texture(512x512)
  Layer: Realm A, full
```

Result: two independent `RenderInvocation`s for Realm A, each with its own `render_size_px`.

## 2. Multi-target texture pipeline

Goal: render a scene to texture, process it, then present final output.

```text
Target Texture Scene
  Layer: Game3D

Target Texture Blur
  Layer: Blur2D (samples Scene)

Target Window Main
  Layer: Final2D (samples Blur)
```

`FrameGraph` execution order is inferred as:

```text
Scene -> Blur -> Main
```

## 3. Passes with `require` and `priority`

```ts
engine.graph3d.definePass({
  name: "bright-extract",
  type: "screen",
  input: ["color"],
  output: "bright",
  params: { threshold: "f32" },
  shader: `fn fragment(input: FragmentInput) -> FragmentOutput { /* ... */ }`,
});

engine.graph3d.definePass({
  name: "glow",
  type: "screen",
  input: ["color", "bright"],
  output: "color",
  require: ["bright"],
  params: { intensity: "f32" },
  shader: `fn fragment(input: FragmentInput) -> FragmentOutput { /* ... */ }`,
});

realm.graph3d.use("bright-extract", {
  priority: 20,
  params: { threshold: 1.0 },
});

realm.graph3d.use("glow", {
  priority: 30,
  params: { intensity: 0.8 },
});
```

Expected behavior:

- `bright-extract` executes before `glow` (dependency + priority).
- if `bright` is unavailable, `glow` is skipped due to `require`.
