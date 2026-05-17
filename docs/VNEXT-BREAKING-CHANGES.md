# vNext Breaking Changes

This document consolidates breaking changes introduced by the vNext architecture replace.

## Composition contract

1. `TargetKind` public API is restricted to:
- `window`
- `texture`

2. Removed public composition concepts:
- `WidgetRealmViewport`
- `RealmPlane`
- `Connector`
- `Present`
- `Surface` as public composition abstraction

3. `TargetLayer` is the only host-facing realm composition primitive.

## Input contract

1. Target-routed pointer relay was removed.
2. Pointer input is global stream only.
3. Legacy target/listener UI input bridge commands were removed.

## UI stack

1. Legacy core-integrated UI stack was removed from runtime/core.
2. `vulfram-realm-ui` crate was removed from workspace.
3. Legacy UI commands/events and docs were removed.

## Material contract

1. Material model is unified under `ShaderMaterial`.
2. Public material payload must use:
- `kind: shader`
- `preset: standard | pbr`
3. Legacy payload forms (`kind: standard|pbr`) are rejected.

## Graph contract

1. Global frame ordering is handled by `FrameGraph`.
2. Internal pass graphs are split by realm kind:
- `Graph3D`
- `Graph2D`
3. Pass behavior is defined by `input`, `output`, `require`, `params`, `priority`.

## Demo/runtime cleanup

1. Legacy demo scenarios were removed/reduced.
2. Dead code from old demo UI/input flow was removed.

## Upgrade expectation

Host integrations must migrate command payloads and architecture assumptions before adopting vNext release tags.
