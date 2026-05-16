# Vulfram Overview

Vulfram is a rendering and systems core written in Rust and exposed to hosts
through ABI-oriented bindings.

Supported host shapes include:

- Node.js
- Lua
- Python
- browser runtimes through WASM
- any environment capable of calling the exported ABI surface

The host drives Vulfram through:

- a small set of exported `vulfram_*` functions
- MessagePack command/response/event buffers

The core remains a black box for GPU, window, platform and render execution.

## 1. Design Goals

- host-agnostic integration
- minimal public surface
- binary communication and low overhead
- strict ownership split between host decisions and core execution
- order-independent resource linking with fallback behavior

## 2. Responsibility Split

### Host

The host is responsible for:

- world/game/application logic
- logical ID generation and validity
- command batch creation
- driving `vulfram_tick`
- reading responses, events and profiling data

The host does not:

- create `wgpu` objects
- create internal composition runtime tables directly
- talk directly to platform window APIs through Vulfram internals

### Core

The Rust core is responsible for:

- platform integration
- window/input collection
- GPU bootstrap and render execution
- resource decoding, fallback handling and render caches
- realm composition
- automatic resolution of internal composition tables from host-provided targets

## 3. Main Concepts

### Components

High-level scene participants addressed by host IDs:

- camera
- model
- light
- audio listener/source

They are realm-scoped and updated through commands.

### Resources

Reusable assets/configurations addressed by host IDs:

- geometry
- material
- texture
- environment profile
- render graph resource
- UI theme/font/image

### Internal Composition Tables

These are core-owned tables:

- internal target/frame composition caches

They are not host-upserted directly. They are derived internally from:

- `Realm`
- `Target`
- `TargetLayer`

## 4. Realms, Targets and Auto-Graph

The current composition model is:

- `Realm`
  - execution scope and bound `render_graph_id`
- `Target`
  - logical output anchor such as `Window` or `Texture`
- `TargetLayer`
  - binds one realm to one target with layout/composition metadata

From those host-visible maps, the runtime derives:

- render invocations per layer
- target dependency ordering in the frame graph
- `TargetGraph` and `RealmGraph` diagnostics

Practical rule:

- the host owns `RealmId`, `TargetId`, and all scene/resource IDs
- the core owns physical runtime handles and caches

## 5. Render Graphs

Render graphs are global resources stored by logical `render_graph_id`.
Each realm can bind one graph.

Important properties:

- graph resources are host-defined
- graph validation and compiled plan caching are core-side
- graphs are realm-scoped by binding, not window-scoped
- invalid or missing graphs fall back safely by realm kind

## 6. Resource Linking and Fallbacks

Resources may be created out of order.

Examples:

- a model may reference geometry/material that do not exist yet
- a material may reference a texture that has not been uploaded yet

When references are missing, the core uses fallbacks until the real resource
becomes available.

This keeps host orchestration decoupled from upload order.

## 7. Visibility and Ordering

Visibility is filtered with `u32` bitmasks.

- camera layer mask
- component layer mask
- future light layer mask

Common visibility rule:

```text
visible when (layerMaskCamera & layerMaskComponent) > 0
```

Per-camera ordering:

- opaque and masked draws are sorted to reduce state changes
- transparent draws are sorted by depth

## 8. Current Architecture Direction

After the recent refactor, the workspace direction is:

- `vulfram-runtime`
  - integration root and command lifecycle
- `vulfram-render`
  - rendering policy, render graph policy and increasing ownership of
    auto-graph planning
- `vulfram-realm-core`
  - realm composition semantics and DTOs

The main unresolved design area is state ownership:

- `UniversalState` is still broader than realm composition alone
- the long-term goal is to split it into smaller sub-states by ownership rather
  than moving the current aggregate wholesale into `vulfram-realm-core`
