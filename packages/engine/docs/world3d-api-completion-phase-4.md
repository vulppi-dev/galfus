# World3D API Completion (Phase 4)

This document records the completion of missing `World3D` input/diagnostics
wrappers that already existed in internal state queries.

## Goal

- Expose a complete and consistent `3D`-prefixed public query surface.
- Keep behavior aligned with existing `state-queries` semantics.

## Added APIs

Keyboard edge:

- `is3DKeyJustPressed(worldId, keyCode)`
- `is3DKeyJustReleased(worldId, keyCode)`

Window state:

- `get3DWindowPosition(worldId)`
- `is3DWindowFocused(worldId)`
- `was3DWindowResized(worldId)`
- `get3DWindowScaleFactor(worldId)`

Gamepad:

- `get3DGamepadEvents(worldId)`
- `get3DConnectedGamepads(worldId)`
- `get3DGamepadAxis(worldId, gamepadId, axis)`
- `is3DGamepadButtonPressed(worldId, gamepadId, button)`

Diagnostics:

- `get3DLastSystemError(worldId)`
- `get3DSystemEvents(worldId)`

## Implementation Notes

- APIs were added in `src/engine/world/world3d-input.ts`.
- Backed by `src/engine/input/api.ts` (which now re-exports the required
  internal query functions).

## Validation

Typecheck passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
