# Input Inventory (Phase 0 Baseline)

This inventory captures the current input pipeline and access points before
structural extraction of input modules.

## Scope

- Runtime event routing and mirroring.
- ECS state ownership for input/window/gamepad/system/ui event snapshots.
- Read/query points used by public APIs (`World3D`, `WorldUI`, systems).

## Source of Events

- Event routing: `src/engine/bridge/dispatch.ts` (`routeEvents`).
- Tick receive order: `src/engine/api.ts` (`tick`).

World routing index keys used by event fanout:

- `windowId`
- `realmId`
- `targetId`

## Write Paths (State Mutation)

Primary writer for mirrored snapshots:

- `src/engine/systems/input-mirror.ts` (`InputMirrorSystem`)

State components created/mutated at world entity `0`:

- `InputState`
- `WindowState`
- `GamepadState`
- `SystemEventState`
- `UiEventState`

Reset-at-frame-start behavior is defined in:

- `docs/input-frame-contract.md`

Additional read usage inside systems:

- `src/engine/systems/ui-bridge.ts` reads `InputState` for UI behaviors.

## Read Paths (Queries)

Canonical query layer:

- `src/engine/world/entities/state-queries.ts`

Categories exposed there:

- Keyboard (`isKeyPressed`, edge queries)
- Pointer global/target snapshots
- Scroll and IME
- Window state and transients
- Gamepad state/events
- System events and last error
- UI events

## Public API Mapping

Primary facade:

- `src/engine/world/world3d.ts`

Current state:

- Most pointer/window/keyboard base queries are wrapped with `3D` prefix.
- Missing wrappers identified in Phase 0 were completed in Phase 4:
  - key edge
  - window position/focus/resize/scale
  - gamepad queries
  - last system error and full system events

Related docs:

- `docs/events-input.md`
- `docs/input-frame-contract.md`

## Boundaries for Phase 1+

Guaranteed by this baseline:

- No behavior change introduced in Phase 0.
- Existing snapshot semantics remain authoritative until extraction lands.

Planned extraction boundaries:

- `src/engine/input/core/*`: global input normalization/state policy.
- `src/engine/input/routing/*`: routed pointer snapshot policy by world/target.
