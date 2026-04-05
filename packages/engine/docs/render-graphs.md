# Render Graphs

The core now exposes render graph catalog commands in the same command batch used by the engine.

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
- Reserved default IDs from the core cannot be disposed or overwritten.
- `two-d` realms can bind only UI-compatible graphs (`passId = "ui"` in all nodes).
