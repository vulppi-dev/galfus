# Render Graphs

The core now exposes render graph catalog commands in the same command batch used by the engine.

On the TypeScript side, the public contract is intentionally narrower than the raw Rust union in a few places:

- `graphId`, `nodeId`, `resId`, and related logical ids are numeric in the engine API.
- `params` remains a string-keyed value map at the wire level, but `RenderGraphNode`, `RenderGraphDesc`, and `CmdRenderGraphUpsertArgs` are now generic so host code can specialize param types per graph or per pass family.

## Available Commands

- `cmd-render-graph-upsert`: create or update a graph by `renderGraphId`.
- `cmd-render-graph-dispose`: remove a graph from catalog.
- `cmd-render-graph-list`: list current catalog entries.
- `cmd-realm-render-graph-bind`: bind a realm to a graph id.

## 3D World API

Use the `world3d` wrappers:

- `upsert3DRenderGraph(worldId, args)`
- `dispose3DRenderGraph(worldId, args)`
- `list3DRenderGraphs(worldId, args?)`
- `bind3DRealmRenderGraph(worldId, args)`

## UI World API

Use the `world-ui` wrappers:

- `upsertUIRenderGraph(worldId, args)`
- `disposeUIRenderGraph(worldId, args)`
- `listUIRenderGraphs(worldId, args?)`
- `bindUIRealmRenderGraph(worldId, args)`

## Notes

- `renderGraphId` uses host-managed numeric IDs.
- `LogicalId` in the engine API is also numeric, even though the core model supports a broader internal union.
- Reserved default IDs from the core cannot be disposed or overwritten.
- `two-d` realms can bind only UI-compatible graphs (`passId = "ui"` in all nodes).
- Default `params` typing is `Record<string, boolean | number | string>`, and can be specialized incrementally by consumers:

```ts
type BloomParams = {
  threshold: number;
  intensity: number;
};

const graph: RenderGraphDesc<BloomParams> = {
  graphId: 10,
  nodes: [
    {
      nodeId: 1,
      passId: 'bloom',
      params: {
        threshold: 1.0,
        intensity: 0.8
      }
    }
  ],
  edges: []
};
```
