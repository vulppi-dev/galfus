# Render Graph

Render graphs are global resources stored by logical `render_graph_id` and
bound per realm.

They are:

- host-defined
- core-validated
- cached by compiled plan/hash
- realm-scoped by binding

They are not window-scoped and do not replace the auto-graph composition model.

## Flow

1. Host upserts a render graph resource.
2. Core validates the graph.
3. Core compiles or reuses a cached execution plan.
4. A realm binds to that graph through `render_graph_id`.
5. Missing or invalid graphs fall back safely by realm kind.

## Relationship to Auto-Graph

Render graph and auto-graph solve different problems:

- render graph
  - defines pass/resource execution inside a realm
- auto-graph
  - defines how realms are composed across targets/windows/surfaces

The host participates in both, but through different APIs:

- render graph resource commands
- realm/target/target-layer commands

## Core Rules

- graph resources live in a global catalog
- realm binding chooses which graph a realm executes
- validation and plan caching are core-side
- the runtime may use fallback graphs when the requested graph is absent or
  incompatible with the realm kind
