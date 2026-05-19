# Render Graph

In vNext, render graph behavior is split into two scopes.

## 1. Global frame scope: `FrameGraph`

`FrameGraph` schedules target execution for one frame.

It tracks:

- active targets
- layers per target
- render invocations
- target textures produced this frame
- texture reads required by realms
- dependency order between targets

Rules:

- ordering is deterministic
- cycles are allowed, with cyclic reads resolved from previous-frame cache

## 2. Per-realm scope: `Graph3D` and `Graph2D`

Each realm kind executes an internal pass graph per invocation:

- `Graph3D` for 3D realms
- `Graph2D` for 2D realms

Pass catalogs are separated. A pass definition belongs to exactly one graph kind.

## Pass model

A pass definition declares:

- `name`
- `type`: `screen | draw | compute`
- `input`
- `output`
- `require`
- `params`
- `shader`

Pass usage (`use`) declares runtime intent:

- `priority`
- `params` values
- optional `enabled`

## Ordering rules inside `Graph3D/Graph2D`

Pass order is resolved by:

1. `input/output` dependencies
2. `require`
3. `priority`
4. stable name tie-break for independent nodes

If two passes can produce non-commutative writes on the same output and remain ambiguous, graph compilation fails.

## Shader DSL boundary

Client pass shader code is functional DSL-like WGSL subset.

The core generates physical WGSL details internally:

- entrypoints
- bind groups and bindings
- pipeline layouts
- ping-pong temporaries
- params uniform buffers

This preserves API simplicity and keeps physical binding details private.
