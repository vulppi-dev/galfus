# World Input Adapter (Phase 3)

This document records Phase 3, where world facades were adapted to consume the
new `engine/input` module and input-related APIs were split into dedicated
adapter files.

## Goal

- Reduce direct coupling from world facades to `world/entities` input queries.
- Route world input APIs through `engine/input` adapters.
- Thin `world3d.ts` by splitting input APIs into a dedicated module.

## Implemented Changes

1. New adapter module:
- `src/engine/world/world3d-input.ts`

This file now owns `World3D` input-facing APIs:

- input target listener commands
- target pointer event filtering
- keyboard/IME/window/pointer/scroll queries

2. `World3D` facade split:
- `src/engine/world/world3d.ts` now re-exports from `./world3d-input`
- input API implementations were removed from `world3d.ts`
- `world3d.ts` line count reduced from ~593 to ~398

3. Adapter delegation through `engine/input`:
- `world3d-input.ts` uses `src/engine/input/api.ts` for query access
- `world-ui.ts#getUIEvents` now uses `getUiEvents` from `engine/input/api`

4. Internal query exposure:
- `src/engine/input/queries.ts` added as canonical internal bridge to
  state-query reads used by world adapters.

## Behavior Contract

- No intentional public API behavior changes.
- Existing `World3D` and `WorldUI` signatures remain stable.

## Validation

Typecheck passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
