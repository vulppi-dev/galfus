# Legacy Cleanup and Convergence (Phase 6)

This phase removes redundant legacy paths after the input refactor and aligns
world adapters with the new routing source-of-truth.

## Cleanup Performed

1. World3D routed pointer reads now use routing snapshot API:

- `src/engine/world/world3d-input.ts` switched routed pointer getters to
  `getRoutedPointerSnapshotByWorld(...)` instead of direct target query helpers.

Affected wrappers:

- `get3DPointerTargetPosition`
- `get3DPointerTargetSize`
- `get3DPointerTargetDelta`
- `get3DPointerTargetId`
- `get3DPointerTargetUv`

2. Removed redundant aggregate export path:

- `src/engine/world/entities.ts` no longer re-exports `state-queries`.
- World facades now consume query adapters via `engine/input/*`.

3. Documentation convergence:

- Updated `docs/input-inventory-phase-0.md` to reflect that previously missing
  wrappers are now completed.

## Result

- Reduced ambiguity in routed pointer source-of-truth.
- Less chance of accidental future coupling to legacy query paths.
- Cleaner boundary between:
  - world entity command/intents surface
  - input query/routing adapter surface

## Validation

Typecheck passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
