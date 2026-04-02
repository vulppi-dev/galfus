# Vulfram Internal API

This document is the internal architecture reference for the Rust workspace.
It describes the current crate boundaries, the main runtime state holders, and
the recommended direction for future refactors after the large architecture
split.

## 1. Current Workspace Roles

The workspace is currently organized around these main crates:

- `vulfram-runtime`
  - ABI entry points used by the bindings
  - command/response/event orchestration
  - frame lifecycle and deferred command handling
  - integration layer between platform, realm, render, audio, UI, and resources
- `vulfram-render`
  - WGPU-facing policies and helpers
  - render graph validation/plan cache
  - realm/target planning helpers
  - render target/bootstrap/cache utilities
- `vulfram-realm-core`
  - realm composition semantics
  - `RealmState`, `ConnectorState`, `PresentState`
  - realm graph plan, target graph plan, frame report DTOs
- `vulfram-platform`
  - desktop/browser bootstrap and platform policies
- `vulfram-protocol`
  - host/runtime contracts and codec-friendly DTOs
- `vulfram-types`
  - shared logical IDs and base enums
- `vulfram-input`
  - normalized input routing contracts and helpers
- `vulfram-realm-ui`
  - UI semantic state, documents, interaction and trace planning
- `vulfram-realm-3d`
  - 3D realm sync and pass-facing semantic helpers
- `vulfram-realm-2d`
  - reserved 2D realm contract surface
- `vulfram-audio`
  - audio contracts, sync plans and backends
- `vulfram-bindings-*`
  - host bindings over the runtime ABI
- `vulfram-demo`
  - manual and visual validation harness

Important naming note:

- the bindings still depend on `vulfram-runtime` through the dependency alias
  `vulfram-core` for compatibility of local binding code
- there is no standalone `crates/vulfram-core` crate in the workspace anymore

## 2. Runtime Ownership Model

At the top level, `EngineState` in `vulfram-runtime` is still the integration
root. It owns:

- window/platform integration state
- render manager state
- GPU bootstrap handles (`wgpu::Instance`, `Device`, `Queue`)
- upload/decode/audio runtime services
- the cross-domain world tables grouped in `UniversalState`
- response/event queues and deferred command lifecycle
- per-frame profiling state

This is a practical integration root, not an ideal domain boundary.

### 2.1 `UniversalState` Today

`UniversalState` is currently a realm-centric aggregate, but it is broader than
realm composition alone. Today it contains:

- realm composition tables:
  - `realms`, `surfaces`, `connectors`, `presents`
- target/auto-graph tables:
  - `targets`, `target_layers`, `target_graph_cache`, `auto_links`
- routing/runtime diagnostics:
  - `host_realm_index`, `target_ui_realm_index`, `target_autolink_failures`
  - `input_routing`, `target_listeners`
  - `surface_cache`, `frame_report`
- realm-attached content/resource registries:
  - `realm3d`
  - `render_resources`
  - `render_graphs`, `render_graph_plan_cache`
- UI state:
  - `ui`

Because of that, `UniversalState` should not be moved wholesale into
`vulfram-realm-core` yet. The name suggests "all runtime world state", but its
contents mix multiple domains:

- realm composition
- target routing
- input routing
- UI integration
- render graph catalog
- 3D resource registries

### 2.2 Recommended Split for `UniversalState`

The cleaner direction is to split by ownership instead of moving the current
aggregate as-is:

1. `RealmCompositionState` in `vulfram-realm-core`
   - `realms`
   - `surfaces`
   - `connectors`
   - `presents`
   - `surface_cache`
   - `frame_report`
   - target graph / realm graph report DTOs

2. `TargetRoutingState` in `vulfram-runtime` or a future `vulfram-target`
   - `targets`
   - `target_layers`
   - `target_graph_cache`
   - `auto_links`
   - `host_realm_index`
   - `target_ui_realm_index`
   - `target_autolink_failures`

3. `SceneResourceState` in runtime or realm-specific crates
   - `realm3d`
   - `render_resources`
   - `render_graphs`
   - `render_graph_plan_cache`

4. `UiRuntimeState`
   - `ui`

5. `InputRoutingRuntimeState`
   - `input_routing`
   - `target_listeners`

That gives `realm-core` a sharper purpose: realm composition semantics and
plans, not the whole runtime world.

## 3. Auto-Graph Ownership

Current behavior:

- the host upserts `Target` and `TargetLayer`
- the runtime reconciles these logical maps
- internal `Surface`, `Present`, and `Connector` tables are derived from them
- render execution later consumes the derived realm/surface/connectors state

Recommended ownership:

- host-facing commands stay in `vulfram-runtime`
- graph planning rules belong in `vulfram-render`
- pure data semantics belong in `vulfram-realm-core`

That means the long-term ideal is:

- `vulfram-realm-core`
  - target/realm composition DTOs and table/state types
- `vulfram-render`
  - auto-graph planner and reconciliation planning
  - target/realm composition policy because it directly drives realm ordering,
    surface sizing, composition and WGPU-facing execution constraints
- `vulfram-runtime`
  - command decoding
  - applying the planner result into owned runtime tables
  - emitting diagnostics/events

In other words: the auto-graph should be render-owned in policy, but not
because it "owns WGPU handles". It should be render-owned because it decides
how realms are composed for rendering.

## 4. `vulfram-realm-core` Scope

`vulfram-realm-core` already owns important composition types, but it is still
too monolithic internally.

Recommended internal split:

1. `types.rs`
   - IDs re-exported from `vulfram-types`
   - small enums and common DTO keys

2. `state.rs`
   - `RealmState`
   - `ConnectorState`
   - `PresentState`
   - `AutoLink`
   - `SurfaceCache`

3. `tables.rs`
   - `RealmTable`
   - `ConnectorTable`
   - `PresentTable`
   - shared `TableEntry`

4. `realm_graph.rs`
   - `RealmGraphEdge`
   - `RealmGraphPlan`
   - `RealmGraphPlanner`

5. `target_graph.rs`
   - `TargetId`
   - `TargetKind`
   - `DimensionValue`
   - `TargetLayerLayout`
   - `TargetGraphPlan`
   - `TargetGraphDiff`
   - `TargetGraphPlanner`

6. `report.rs`
   - `FrameReport`
   - `FrameCutEdge`
   - `SurfaceCacheEntry`
   - `TargetLayerReportKey`
   - `TargetAutoLinkFailure`

7. `render_passes.rs`
   - reserved pass IDs/constants only

This split keeps `realm-core` focused on realm composition semantics and makes
it easier to move planner logic in/out without dragging unrelated DTOs.

## 5. Suggested State Organization

The current `EngineState` works, but it is broad. A better medium-term layout is
to make the state tree reflect ownership:

- `EngineState`
  - `platform: PlatformRuntimeState`
  - `gpu: GpuRuntimeState`
  - `runtime: RuntimeLoopState`
  - `world: WorldState`
  - `profiling: ProfilingState`

- `WorldState`
  - `composition: RealmCompositionState`
  - `targets: TargetRoutingState`
  - `scene3d: Realm3dSceneState`
  - `render_resources: RenderResourceState`
  - `ui: UiRuntimeState`
  - `input_routing: InputRoutingRuntimeState`

- `GpuRuntimeState`
  - `instance`
  - `caps`
  - `device`
  - `queue`
  - `surface_targets`
  - `present_sizes_cache`
  - `bootstrap_support`

- `RuntimeLoopState`
  - command queue
  - response queue
  - event queue
  - deferred command metadata
  - frame clock/index

This organization gives clearer rules:

- composition data does not own input/UI registries
- GPU caches do not live beside domain semantics by accident
- command lifecycle stays separate from world data

## 6. Documentation Truths for Contributors

These are the current rules the rest of the docs should follow:

- `Surface`, `Present`, and `Connector` are internal runtime tables
- the host owns `RealmId`, `TargetId`, resource IDs, component IDs and window IDs
- the host does not upsert `Surface`, `Present`, or `Connector` directly
- the auto-graph derives runtime composition from `Target` + `TargetLayer`
- render graph resources are global catalog entries bound per realm through
  `render_graph_id`
- `vulfram-runtime` is the integration root of the Rust side today
- `vulfram-render` owns rendering policy and should increasingly own
  auto-graph planning policy

## 7. Practical Refactor Order

If we continue the refactor, a low-risk order is:

1. split `vulfram-realm-core/src/lib.rs` into smaller files without changing APIs
2. extract `UniversalState` into nested sub-states inside `vulfram-runtime`
3. move pure auto-graph planning from runtime to `vulfram-render`
4. keep runtime responsible only for command application and event emission
5. only then consider whether a smaller composition state should move fully into
   `vulfram-realm-core`
