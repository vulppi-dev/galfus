# Phase 0 Inventory (Impact Map)

## Legacy Architecture Hotspots
- Realm composition internals still modeled around `Surface/Present/Connector` in:
  - `crates/vulfram-realm-core`
  - `crates/vulfram-render/src/realm_planner/*`
  - `crates/vulfram-runtime/src/core/target/*` and render orchestration
- Legacy target semantics still present in docs/UI contracts (`WidgetRealmViewport`, `RealmPlane`).

## Public API / Protocol Surface Impact
- Target commands and types:
  - `cmd-target-upsert`
  - `cmd-target-layer-upsert`
  - `packages/engine/src/types/cmds/target.ts`
- Pointer relay and target-listener commands:
  - `cmd-input-target-listener-upsert`
  - `cmd-input-target-listener-dispose`
  - `cmd-input-target-listener-list`
  - TS command unions in `packages/engine/src/types/cmds/index.ts`
- Pointer event payload routing fields in host types/docs (`pointerTarget*`, target-relative fields, trace chain).

## Runtime Areas Impacted
- Tick pipeline integration points:
  - `crates/vulfram-runtime/src/core/tick.rs`
- Input routing/listener path:
  - `crates/vulfram-runtime/src/core/input/routing.rs`
  - `crates/vulfram-runtime/src/core/input/listeners/*`
- Command dispatch/defer/response mappings:
  - `crates/vulfram-runtime/src/core/cmd/processing/*`
  - `crates/vulfram-runtime/src/core/cmd.rs`

## Engine Host Areas Impacted
- Input mirror and routed pointer handling:
  - `packages/engine/src/engine/systems/input-mirror.ts`
  - `packages/engine/src/engine/input/routing/*`
  - `packages/engine/src/engine/world/world3d-input.ts`
- Command type unions and input command DTOs:
  - `packages/engine/src/types/cmds/*`
- Pointer event typings:
  - `packages/engine/src/types/events/pointer.ts`

## Documentation Areas Impacted
- Core architecture docs:
  - `docs/ARCH.md`, `docs/REALM-ARCH.md`, `docs/API.md`
- Command docs:
  - `docs/cmds/*target*`, `docs/cmds/*input-target-listener*`
- Engine input docs under `packages/engine/docs/*input*`.
