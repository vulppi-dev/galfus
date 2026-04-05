# Input Architecture (Final)

This document summarizes the final input architecture after the refactor.

## High-Level Structure

```text
src/engine/input/
  core/
    state.ts
    keyboard.ts
    gamepad.ts
    pointer.ts
  routing/
    pointer-routing.ts
    world-bindings.ts
    types.ts
  queries.ts
  api.ts
```

## Responsibilities

### Input Core (`input/core`)

Owns global input state lifecycle and normalization:

- frame reset policy
- keyboard state and edge transitions
- global pointer state (position/delta/buttons/scroll/window context)
- gamepad mirrored state/events

### Input Routing (`input/routing`)

Owns routed pointer snapshot semantics:

- `pointerTargetId`
- `pointerTargetPosition`
- `pointerTargetDelta`
- `pointerTargetUv`
- `pointerTargetSize`

Also exposes internal routed snapshot reads by:

- world scope
- target scope

### Input Queries (`input/queries.ts`)

Canonical bridge for read access to mirrored state queries, consumed by world
facades/adapters.

## Runtime Integration

### `InputMirrorSystem`

Flow per pointer event:

1. apply global pointer handling (`input/core/pointer.ts`)
2. apply routed pointer handling (`input/routing/pointer-routing.ts`)

Frame-reset behavior follows `docs/input-frame-contract.md`.

### World Facades

- `world3d-input.ts` is the input-facing `World3D` adapter.
- `world3d.ts` re-exports `world3d-input` plus scene/resource APIs.
- `world-ui.ts` reads UI events through `engine/input/api`.

## Source-of-Truth Rules

- Global pointer data comes from input core path.
- Routed pointer data comes from routing snapshot path.
- Public `World3D` queries are wrappers over these adapter/query layers.

## Design Outcomes

- Clear separation between global input and contextual routing.
- Reduced facade coupling to entity-level internals.
- Smaller, more maintainable world facade files.
- Consistent public query surface for keyboard/window/gamepad/diagnostics.
