# Input Usage Guide

This guide shows how to consume input queries from `@vulfram/engine/world3d`.

## Keyboard

```ts
if (World3D.is3DKeyPressed(worldId, World3D.KeyCode.KeyW)) {
  // continuous movement
}

if (World3D.is3DKeyJustPressed(worldId, World3D.KeyCode.Space)) {
  // one-shot toggle
}
```

## Pointer

```ts
const globalDelta = World3D.get3DPointerDelta(worldId);
const targetDelta = World3D.get3DPointerTargetDelta(worldId);
const targetId = World3D.get3DPointerTargetId(worldId);

if (World3D.is3DPointerButtonJustPressed(worldId, 0)) {
  // left-click edge
}
```

## Window

```ts
const size = World3D.get3DWindowSize(worldId);
const pos = World3D.get3DWindowPosition(worldId);
const focused = World3D.is3DWindowFocused(worldId);

if (World3D.was3DWindowResized(worldId)) {
  // react once per resize frame
}
```

## Gamepad

```ts
const connected = World3D.get3DConnectedGamepads(worldId);
const events = World3D.get3DGamepadEvents(worldId);

if (connected.length > 0) {
  const id = connected[0]!.gamepadId;
  const lx = World3D.get3DGamepadAxis(worldId, id, 0);
  const aPressed = World3D.is3DGamepadButtonPressed(worldId, id, 0);
}
```

## Diagnostics

```ts
const lastError = World3D.get3DLastSystemError(worldId);
const systemEvents = World3D.get3DSystemEvents(worldId);
```

## Frame-Semantics Notes

- Edge queries (`JustPressed`, `JustReleased`) are frame-local.
- Scroll and pointer deltas are frame-local snapshots.
- Routed pointer values are based on routed pointer event snapshots.

See:

- `docs/input-frame-contract.md`
- `docs/input-architecture-final.md`
