# Input Frame Contract

This document defines the current frame semantics for mirrored input/window
state. It reflects the behavior of the active runtime pipeline and is intended
to be the baseline for the incremental input refactor.

## Frame Order

Per `tick(timeMs, deltaMs)`, the order is:

1. Receive events from core (`vulframReceiveEvents`) and route to worlds.
2. Receive command responses from core (`vulframReceiveQueue`) and route.
3. Execute systems for each world in order:
   - `input`
   - `update`
   - `preRender`
   - `postRender`
4. Collect/send commands (`vulframSendQueue`).
5. Execute core tick (`vulframTick`).

## Reset Semantics (Start of `InputMirrorSystem`)

At the beginning of each world `input` step:

- `keysJustPressed`, `keysJustReleased` are cleared.
- `pointerJustPressed`, `pointerJustReleased` are cleared.
- `pointerDelta` resets to `[0, 0]`.
- `pointerTargetDelta` resets to `[0, 0]` only when a routed target exists;
  otherwise it becomes `undefined`.
- `scrollDelta` resets to `[0, 0]`.
- `imeCommitText` resets to `undefined`.
- window transient flags reset:
  - `resizedThisFrame = false`
  - `movedThisFrame = false`
  - `focusChangedThisFrame = false`
  - `closeRequested = false`
- `eventsThisFrame` buffers reset for gamepad/system/ui.

## Event Application Semantics

- Keyboard:
  - press adds to `keysPressed`; first press in frame adds to `keysJustPressed`.
  - release removes from `keysPressed` and adds to `keysJustReleased`.
- Pointer:
  - global delta is computed from latest event position minus previous position.
  - target-relative snapshot (`pointerTarget*`) is updated from routed pointer
    event payload (`positionTarget`, `trace.targetId`, `trace.uv`).
  - target delta is frame-local and preserves continuity only for same target id.
- Scroll:
  - line delta is mirrored as-is.
  - pixel delta is normalized by `/20` (approximate line conversion).
- Window:
  - close request is one-frame truthy.
  - resize/focus/move transients are one-frame truthy.

## Snapshot Moment

The routed pointer snapshot (`pointerTargetId`, `pointerTargetPosition`,
`pointerTargetDelta`, `pointerTargetUv`) is considered captured at the moment
the routed pointer event is mirrored by `InputMirrorSystem` in the `input`
system step.

## Non-Goals for Phase 0B

- No runtime behavior changes.
- No event-routing algorithm changes.
- No public API changes.
