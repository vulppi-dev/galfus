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

## Wrap Layout

```text
layout: {
  direction: "row",
  wrap: true,
  wrapLimit: 180,
  gap: 6
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
