# Input Hardening (Phase 7)

This checklist captures final hardening validation status.

## Scenarios

1. Focus changes (`focused`, edge behavior)
2. Window resize (`was3DWindowResized`, size/scale consistency)
3. Pointer capture lock/unlock (`get3DWindowPointerCaptureState`)
4. UI/3D interaction switching (routed target + click/scroll)
5. IME states (`is3DImeEnabled`, preedit/commit)
6. Gamepad state/events and query consistency
7. Diagnostics visibility (`get3DLastSystemError`, `get3DSystemEvents`)

## Validation Status

Automated validation:

- Typecheck passed for:
  - `packages/engine`
  - `packages/camera-control`
  - `packages/gltf-loader`
  - `apps/demos`

Manual smoke:

- Not executed in this phase inside this environment.
- Recommended target scenario: `apps/demos/src/007.demo.ts` because it exercises
  UI + 3D + pointer routing + pointer lock + keyboard + diagnostics.

## Result

- Architecture and API are stabilized and documented.
- Remaining runtime confidence step is manual smoke execution in a windowed host.
