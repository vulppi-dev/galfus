# @galfus/camera-control

Standard camera controllers for `@galfus/engine`.

## Included Controllers

- `createOrbitController`
- `createSpectatorController`
- `createFirstPersonController` (cinematic, without physics/collision)
- `createThirdPersonController`
- `createTopViewController`

## Core Behavior

- Pointer input is internally coupled for all controllers.
- Pointer deltas used for look/rotation follow the raw pointer movement reported
  by the engine in window/surface space, so camera speed stays stable when the
  viewport or window size changes.
- Raw pointer deltas are still preserved for gestures such as pan/zoom that should
  keep window-space behavior.
- Controller state and transform updates stay in `@galfus/engine/math` `vec3` / `quat` form all the way to `@galfus/engine`.
- Current pointer gesture model is mouse-based (`left/middle/right`).
- Every controller config accepts `pointerDeltaSensitivity` (`1` default) to scale
  pointer-driven rotation/look speed.
- Every controller config accepts `invertPointerX` / `invertPointerY` (`false` by
  default) to invert pointer axis response.
- `Orbit`, `ThirdPerson`, and `TopView` accept `zoomSensitivity` (`1` default) to
  scale zoom response (pointer gesture and `toZoom()` impulse).
- `translationStrategy` and `easing` are optional in every controller config.
- Without `translationStrategy`/`easing`, movement is linear.
- All controllers support `lookAt(position, weight?)`.
  - Negative weight rotates by the longest arc.
- `Orbit` gesture split:
  - rotate: right button or left button (without middle button)
  - pan: middle button (without right button)
- `Orbit` supports enable/disable flow:
  - `enable()`
  - `disable()`
  - `setEnabled(boolean)`
  - `isEnabled()`
- `TopView` supports focus lock:
  - `focusLocked` (config)
  - `setFocusLocked(boolean)`
  - `isFocusLocked()`

## Motion Actions (Spectator / FirstPerson)

Use weighted triggers (`1.0` default):

```ts
controller.pressForward(); // persistent while pressed
controller.releaseForward();

controller.toForward(2.0); // one-frame impulse
controller.look(3, -2, 1.5); // weighted look delta
```

Negative weights are accepted:

```ts
controller.toForward(-1.0); // opposite direction
controller.lookAt([0, 1, 0], -1); // longest-arc look rotation
```
