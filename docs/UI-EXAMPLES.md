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
CmdTargetUpsert { targetId: 9005, kind: "texture", sizeOverride: [256,256] }
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

```text
CmdTargetUpsert { targetId: 9204, kind: "texture", sizeOverride: [420,260] }
CmdTextureBindTarget { windowId: 1, textureId: 1400, targetId: 9204 }
CmdMaterialCreate { materialId: 1212, kind: "standard", options: { surfaceType: "transparent", baseTexId: 1400, emissiveTexId: 1400 } }
CmdModelCreate { modelId: 1303, geometryId: <plane>, materialId: 1212 }
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
CmdTargetUpsert { targetId: 9201, kind: "panel-embed", sizeOverride: [640,720] }
CmdTargetUpsert { targetId: 9202, kind: "viewport-embed", sizeOverride: [640,720] }
CmdTargetBindUpsert { realmId: <uiRealm>, targetId: 9201, layout: { rect: [0,0,640,720], zIndex: 1 } }
CmdTargetBindUpsert { realmId: <viewRealm>, targetId: 9202, layout: { rect: [640,0,640,720], zIndex: 0 } }
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
