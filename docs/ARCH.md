# Vulfram Architecture and Lifecycle

This document explains the current runtime architecture, the main ownership
boundaries, and the expected host/core lifecycle.

## 1. High-Level Architecture

Conceptual flow:

```text
Host -> commands/uploads -> vulfram-runtime -> platform/render/audio/UI subsystems -> GPU/OS
```

Current crate roles:

- `vulfram-runtime`
  - integration root
  - command processing
  - frame loop and deferred lifecycle
- `vulfram-render`
  - rendering policy and WGPU-facing helpers
  - render graph validation/cache
  - realm/target planning helpers
- `vulfram-realm-core`
  - realm composition semantics and reports
- `vulfram-platform`
  - platform-specific bootstrap/policies

## 2. Ownership Boundaries

### Host-visible IDs

The host owns and manages:

- window IDs
- realm IDs
- target IDs
- camera/model/light IDs
- resource IDs
- UI IDs
- upload buffer IDs

### Core-owned Internal Tables

The core owns:

- `Surface`
- `Present`
- `Connector`
- GPU resources/handles
- render targets and compiled plans

The host does not create internal composition tables directly.

## 3. Auto-Graph

The current auto-graph model is:

- host upserts `Target`
- host upserts `TargetLayer`
- runtime reconciles logical maps
- core derives `Surface`, `Present`, and `Connector`
- render execution consumes the derived composition

Design direction:

- auto-graph policy belongs conceptually to `vulfram-render`
- application of commands and emission of diagnostics stay in `vulfram-runtime`
- realm composition DTOs/state belong in `vulfram-realm-core`

Reasoning:

- auto-graph is not just routing metadata
- it determines realm composition, sizing, overlay ordering and renderability
- those are render-policy concerns, even when the runtime still applies them

## 4. Runtime State Today

`EngineState` is still the top integration root.

Broadly, it contains:

- platform/window state
- GPU bootstrap state
- render manager state
- audio/upload/decode services
- `UniversalState`
- runtime queues/deferreds
- profiling

`UniversalState` is currently a large aggregate that mixes:

- realm composition tables
- target routing / auto-graph state
- interaction state
- 3D scene/resource registries
- render graph catalogs
- frame diagnostics

That makes it useful operationally, but not a clean domain boundary.

After the current refactor phase, ownership inside `vulfram-runtime` is split
more explicitly even though `UniversalState` still aggregates the sub-states:

- `core/realm/state.rs`
  - `RealmCompositionState`
- `core/target/state.rs`
  - `TargetRoutingState`
- `core/input/state.rs`
  - `InteractionRuntimeState`
- `core/render/state/mod.rs`
  - `SceneRuntimeState`
  - realm-attached render/resource registries

## 5. Recommended State Shape

The preferred medium-term organization is:

```text
EngineState
  platform
  gpu
  runtime_loop
  world
  profiling

WorldState
  composition
  targets
  interaction
  scene
```

Key guideline:

- move by ownership, not by convenience
- do not move the current `UniversalState` wholesale into `vulfram-realm-core`

## 6. Realm-Core Scope

`vulfram-realm-core` should be the home of:

- realm composition state/types
- graph/report DTOs
- pure planners with no runtime side effects

It should not become a container for:

- UI runtime state
- input listener stores
- render graph catalogs
- texture/material registries
- command queues or runtime services

## 7. Lifecycle

### Startup

1. Host loads the binding/library.
2. Host calls `vulfram_init()`.
3. Runtime initializes core state and platform integration roots.
4. GPU device/queue are created lazily when the first compatible window/surface
   is ready.

### Frame

1. Host prepares logical state and uploads if needed.
2. Host sends command batch through `vulfram_send_queue()`.
3. Host calls `vulfram_tick(time, delta_time)`.
4. Runtime:
   - processes ready commands
   - retries deferred commands when applicable
   - refreshes auto-graph derived state
   - renders realms according to realm graph order
   - collects events, responses and profiling
5. Host receives responses/events.

### Shutdown

1. Host calls `vulfram_dispose()`.
2. Runtime tears down windows, render state, audio and runtime tables.

## 8. Documentation Truths

The rest of the documentation should assume:

- `Surface`, `Present`, and `Connector` are internal-only runtime tables
- `Target` and `TargetLayer` are the host-facing composition API
- render graphs are global resources bound per realm
- `vulfram-runtime` is the current integration root
- `vulfram-render` should increasingly own auto-graph planning policy
