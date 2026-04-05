# Input Routing Extraction (Phase 2)

This document records Phase 2, where routed pointer state management was
isolated from input core handling.

## Goal

- Keep global input processing in `input/core`.
- Move contextual pointer target routing to `input/routing`.
- Preserve runtime semantics.

## Implemented Split

Global pointer responsibilities (`input/core/pointer.ts`):

- Pointer global position/delta (`pointerPosition`, `pointerDelta`)
- Pointer buttons and edges
- Pointer window context (`pointerWindowId`, `pointerWindowSize`)
- Scroll delta normalization

Routed pointer responsibilities (`input/routing/pointer-routing.ts`):

- Routed target id (`pointerTargetId`)
- Routed target position (`pointerPositionTarget`)
- Routed target delta (`pointerTargetDelta`)
- Routed target UV (`pointerTargetUv`)
- Routed target size (`pointerTargetSize`)
- Routed state clear/reset helpers

## Internal Read APIs

Added in `input/routing/world-bindings.ts`:

- `getRoutedPointerSnapshotByWorld(worldId)`
- `getRoutedPointerSnapshotByTarget(worldId, targetId)`

These provide internal routed-snapshot reads by world and target scope.

## Integration

`InputMirrorSystem` now applies pointer events in two passes:

1. `applyPointerEvent(...)` from `input/core`
2. `applyRoutedPointerEvent(...)` from `input/routing`

This keeps behavior unchanged while clarifying ownership boundaries.

## Validation

Typecheck passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
