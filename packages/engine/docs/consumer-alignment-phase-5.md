# Consumer Alignment (Phase 5)

This phase aligns dependent consumers with the refactored input architecture,
focusing on demo usage and validation paths.

## Scope

- Validate that `@galfus/camera-control` remains compatible with the new
  `World3D` input adapter structure.
- Update demo usage to exercise newly exposed `World3D` query APIs.

## Changes Applied

Demo updates:

- `apps/demos/src/007.demo.ts`
  - Replaced key-latch toggles with edge-trigger APIs:
    - `is3DKeyJustPressed(..., KeyL)`
    - `is3DKeyJustPressed(..., Space)`
  - Switched UI resize flow to `was3DWindowResized(worldId)`.
  - Added debug usage for newly exposed window/gamepad/diagnostics APIs:
    - `get3DWindowPosition`
    - `is3DWindowFocused`
    - `get3DWindowScaleFactor`
    - `get3DConnectedGamepads`
    - `get3DGamepadEvents`
    - `get3DGamepadAxis`
    - `is3DGamepadButtonPressed`
    - `get3DLastSystemError`

Camera-control package:

- No code changes required in this phase.
- Compatibility verified by typecheck against workspace `@galfus/engine`.

## Validation

Typecheck passing for:

1. `packages/engine`
2. `packages/camera-control`
3. `packages/gltf-loader`
4. `apps/demos`
