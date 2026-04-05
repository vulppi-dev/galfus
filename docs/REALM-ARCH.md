# Realm Composition Architecture

This document records the current realm-composition model and the recommended
refactor direction for `vulfram-realm-core`.

It replaces the old "phase 0 planning" description as the architecture
reference for realms, surfaces, presents, connectors and reports.

## 1. Current Model

### Host-visible inputs

- `Realm`
- `Target`
- `TargetLayer`
- `render_graph_id` bound per realm

### Core-owned derived tables

- `Surface`
- `Present`
- `Connector`

### Runtime diagnostics

- `RealmGraphPlan`
- `TargetGraphPlan`
- `FrameReport`
- auto-link failure records

## 2. Composition Rules

Each `TargetLayer(realm -> target)` participates in runtime composition.

Current practical behavior:

- one realm chooses one primary target deterministically
- the primary target defines the realm output surface characteristics
- host window roots create `Present`
- non-root window composition, viewport composition and realm planes create
  `Connector`
- target graph and realm graph diagnostics are refreshed from these maps

## 3. Ownership Rules

### Host owns

- `RealmId`
- `TargetId`
- resource/component IDs
- correctness/uniqueness of logical IDs

### Core owns

- `SurfaceId`
- `PresentId`
- `ConnectorId`
- auto-link reconciliation
- cycle-breaking caches and reports

## 4. What Belongs in `vulfram-realm-core`

`vulfram-realm-core` should own realm-composition semantics:

- `RealmState`
- `ConnectorState`
- `PresentState`
- composition tables
- realm graph and target graph DTOs/plans
- frame report DTOs
- small shared composition math/value types

It should not own broader runtime state such as:

- UI document state
- input routing captures/listeners
- texture/material registries
- render graph catalogs
- WGPU caches/targets
- 3D semantic state that belongs to specialized realm crates such as
  `vulfram-realm-3d`

## 5. Internal Split Recommendation

Recommended file/module layout:

```text
vulfram-realm-core/
  types.rs
  state.rs
  tables.rs
  realm_graph.rs
  target_graph.rs
  report.rs
  render_passes.rs
```

Why:

- composition state becomes easier to audit
- planners stop sharing a mega-file with unrelated DTOs
- future extraction of auto-graph planning policy becomes simpler

## 6. Auto-Graph Boundary

Recommended long-term boundary:

- `vulfram-realm-core`
  - composition state and DTOs
- `vulfram-render`
  - auto-graph planning policy
  - realm ordering/composition rules related to rendering
  - layer sync decisions for derived internal links
- `vulfram-runtime`
  - command handling
  - applying planner results
  - emitting events/diagnostics

This boundary is preferable to putting the whole auto-graph in runtime because
the planner exists to support render composition, not generic command routing.

## 7. UniversalState Decision

`UniversalState` should not move into `vulfram-realm-core` in its current form.

Reason:

- it contains much more than realm composition
- moving it as-is would make `realm-core` a misleading "misc runtime state"
  crate

Safer direction:

- extract a smaller composition-only state into `vulfram-realm-core`
- keep runtime-specific aggregates in `vulfram-runtime`

## 8. Target State Recommendation

A practical future split is:

- `RealmCompositionState`
  - realm/surface/present/connector/surface-cache/frame-report
- `TargetRoutingState`
  - targets/target-layers/target-graph-cache/auto-links/indexes/failures
- `SceneResourceState`
  - scene/resource registries instantiated by runtime, with semantic 3D types
    defined in `vulfram-realm-3d`
- `RenderCatalogState`
  - render graph catalog + plan cache
- `UiRuntimeState`
  - UI runtime
- `InputRoutingRuntimeState`
  - captures/focus/listeners

That preserves a clean meaning for each state tree.
