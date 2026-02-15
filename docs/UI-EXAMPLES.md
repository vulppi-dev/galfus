# UI Examples (Host Side)

This document shows minimal command sequences for the host to drive UI.

## Create a UI Realm + Document

1. Create a `TwoD` realm
2. Create a UI document bound to that realm
3. Apply ops to build the tree

```text
CmdRealmCreate { kind: "two-d", windowId: 1 }
CmdUiDocumentCreate { documentId: 100, realmId: <realmId>, rect: [0,0,800,600] }
CmdUiApplyOps {
  documentId: 100,
  version: 1,
  ops: [
    add { parent: null, node: { id: 1, kind: "container", props: { type: "container", content: { layout: { direction: "column", gap: 8 }}}}}
    add { parent: 1, node: { id: 2, kind: "text", props: { type: "text", content: { text: "Hello UI" }}}}
  ]
}
```

## Image from Target Texture

Bind a texture target to a material and also render it in UI:

```text
CmdTargetUpsert { targetId: 9005, kind: "texture", size: [256,256] }
CmdTextureBindTarget { windowId: 1, textureId: 2001, targetId: 9005 }
CmdUiApplyOps {
  documentId: 100,
  version: 2,
  ops: [
    add { parent: 1, node: { id: 3, kind: "image", props: { type: "image", content: { source: { type: "target", content: 9005 } }}}}
  ]
}
```

## UIPlane in 3D (Target Texture)

Render a UI document into a texture target and apply it to a 3D plane:

- Texture sampling convention for UI targets on 3D materials: **top-left** origin.
- The core normalizes UV orientation using `atlas_scale_bias` for standalone/external textures (no shader-specific flip source).

```text
CmdTargetUpsert { targetId: 9204, kind: "texture", size: [420,260] }
CmdTextureBindTarget { windowId: 1, textureId: 1400, targetId: 9204 }
CmdMaterialUpsert { ...create args... }
CmdModelUpsert { ...create args... }
CmdRealmCreate { kind: "two-d", windowId: 1 }
CmdUiDocumentCreate { documentId: 1510, realmId: <realmId>, rect: [0,0,420,260] }
```

## Wrap Layout

```text
layout: {
  direction: "row",
  wrap: true,
  wrapLimit: 180,
  gap: 6
}
```

## Side-by-Side UI + 3D Viewport

Bind a UI panel on the left and a 3D viewport on the right:

```text
CmdTargetUpsert { targetId: 9201, kind: "ui-plane", windowId: 1 }
CmdTargetUpsert { targetId: 9202, kind: "window", windowId: 1 }
CmdTargetLayerUpsert { realmId: <uiRealm>, targetId: 9201, layout: { left: { unit: "px", value: 0 }, top: { unit: "px", value: 0 }, width: { unit: "px", value: 640 }, height: { unit: "px", value: 720 }, zIndex: 1 } }
CmdTargetLayerUpsert { realmId: <viewRealm>, targetId: 9202, layout: { left: { unit: "px", value: 640 }, top: { unit: "px", value: 0 }, width: { unit: "px", value: 640 }, height: { unit: "px", value: 720 }, zIndex: 0 } }
```

Typical left-panel content:

```text
CmdUiApplyOps {
  documentId: 1500,
  version: 1,
  ops: [
    add { parent: 1501, node: { id: 1504, kind: "input", props: { type: "input", content: { value: "Texto inicial", placeholder: "Digite algo..." }}}}
    add { parent: 1501, node: { id: 1505, kind: "button", props: { type: "button", content: { label: "Adicionar" }}}}
    add { parent: 1501, node: { id: 1506, kind: "button", props: { type: "button", content: { label: "Remover" }}}}
    add { parent: 1501, node: { id: 1507, kind: "button", props: { type: "button", content: { label: "[ ] Habilitar efeito" }}}}
  ]
}
```

## Animation

```text
anim: {
  opacity: { from: 0.0, to: 1.0, durationMs: 300, easing: "ease-in-out" },
  translateY: { from: 8.0, to: 0.0, durationMs: 300 }
}
```

`animComplete` events are emitted when each property finishes.
