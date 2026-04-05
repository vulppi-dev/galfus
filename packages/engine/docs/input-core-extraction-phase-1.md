# Input Core Extraction (Phase 1)

This document records the Phase 1 internal refactor where keyboard, pointer,
and gamepad mirroring logic was extracted from `InputMirrorSystem` into
`src/engine/input/core/*`.

## Goal

- Isolate global input core responsibilities.
- Preserve runtime behavior and frame semantics.
- Prepare stable internal API surface for routing extraction in Phase 2.

## Extracted Modules

- `src/engine/input/core/state.ts`
  - `ensureInputMirrorState(...)`
  - `resetInputMirrorFrame(...)`
- `src/engine/input/core/keyboard.ts`
  - `applyKeyboardEvent(...)`
- `src/engine/input/core/pointer.ts`
  - `applyPointerEvent(...)`
- `src/engine/input/core/gamepad.ts`
  - `applyGamepadEvent(...)`
- `src/engine/input/core/index.ts`
  - internal aggregation exports

## Integration Point

- `src/engine/systems/input-mirror.ts` now delegates keyboard/pointer/gamepad
  operations to `input/core` modules.
- Window/system/ui event handling remains in `InputMirrorSystem` for now.

## Behavior Contract

- No intentional runtime behavior changes in this phase.
- Existing frame reset and event-application semantics remain the same.

## Validation

Typecheck executed and passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
