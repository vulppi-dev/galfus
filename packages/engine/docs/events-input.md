# Events and Input

The engine mirrors input and window events into an `InputState` / `WindowState` component. The high-level API exposes query helpers.

## Input Queries

```ts
isKeyPressed(worldId, keyCode);
isKeyJustPressed(worldId, keyCode);
isKeyJustReleased(worldId, keyCode);

getPointerPosition(worldId);
getPointerWindowSize(worldId);
getPointerDelta(worldId);

getPointerTargetPosition(worldId);
getPointerTargetSize(worldId);
getPointerTargetDelta(worldId);
getPointerTargetId(worldId);
getPointerTargetUv(worldId);

isPointerButtonPressed(worldId, button);
isPointerButtonJustPressed(worldId, button);
getScrollDelta(worldId);

isImeEnabled(worldId);
getImePreeditText(worldId);
getImeCursorRange(worldId);
getImeCommitText(worldId);
```

## Window Queries

```ts
getWindowSize(worldId);
getWindowPosition(worldId);
getWindowScaleFactor(worldId);
getWindowLifecycleState(worldId);
getWindowPointerCaptureState(worldId);

isWindowFocused(worldId);
isWindowCloseRequested(worldId);
wasWindowResized(worldId);
```

Browser note:

- Browser canvas windows expose an explicit `on-canvas-active-change` event in the
  raw window event stream.
- While the canvas is active, action input is routed to the engine and page-level
  wheel/touch/navigation defaults are suppressed by the browser proxy.
- The first click inside the canvas activates it but is not forwarded as a pointer
  action event.

Pointer sizing note:

- `getPointerWindowSize()` reports the root input surface size associated with the
  latest pointer event.
- `getPointerTargetSize()` reports the routed target surface/viewport size when the
  pointer event resolves to a target.

## Per-frame Behavior

- "Just pressed" and "just released" sets are cleared every frame.
- `isWindowCloseRequested()` returns true only for the frame where the event arrives.
- `getScrollDelta()` is reset every frame.
- Detailed frame contract: `docs/input-frame-contract.md`.
- Final architecture: `docs/input-architecture-final.md`.
- Usage examples: `docs/input-usage-guide.md`.
- Baseline read/write inventory: `docs/input-inventory-phase-0.md`.

## Event Types

The raw event types live in `Types`:

- `Types.WindowEvent`
- `Types.PointerEvent`
- `Types.KeyboardEvent`
- `Types.GamepadEvent`
- `Types.SystemEvent`

## World3D Convenience Wrappers

`@vulfram/engine/world3d` re-exports the same semantics with `3D`-prefixed helpers, for example:

- `get3DPointerDelta(worldId)`
- `is3DPointerButtonPressed(worldId, button)`
- `get3DWindowSize(worldId)`
- `get3DWindowLifecycleState(worldId)`
- `get3DWindowPointerCaptureState(worldId)`

Additional wrappers include:

- Keyboard edge:
  - `is3DKeyJustPressed(worldId, keyCode)`
  - `is3DKeyJustReleased(worldId, keyCode)`
- Window state:
  - `get3DWindowPosition(worldId)`
  - `is3DWindowFocused(worldId)`
  - `was3DWindowResized(worldId)`
  - `get3DWindowScaleFactor(worldId)`
- Gamepad:
  - `get3DGamepadEvents(worldId)`
  - `get3DConnectedGamepads(worldId)`
  - `get3DGamepadAxis(worldId, gamepadId, axis)`
  - `is3DGamepadButtonPressed(worldId, gamepadId, button)`
- Diagnostics:
  - `get3DLastSystemError(worldId)`
  - `get3DSystemEvents(worldId)`
