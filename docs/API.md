# Vulfram Internal API (vNext snapshot)

This document describes the active architectural contracts after the vNext replace phases.

## Runtime integration root

`vulfram-runtime` remains the integration root for:

- command processing
- response/event emission
- frame orchestration
- platform and GPU lifetime integration

## Public composition contract

Host composition is defined by:

- `Realm`
- `Target`
- `TargetLayer`

Target kinds are restricted to:

- `window`
- `texture`

Removed from public composition API:

- realm-in-realm target abstractions (`WidgetRealmViewport`, `RealmPlane`)
- public `Connector`/`Present` composition model
- target-routed pointer listener contract

## Render ownership model

- `vulfram-render`: render policy and pass/frame execution policy
- `vulfram-realm-core`: composition DTO/state semantics shared by runtime
- `vulfram-runtime`: applies commands and drives frame loop

## Graph model

Two graph levels are active:

- global `FrameGraph` for target ordering and texture dependencies
- per-invocation `Graph3D`/`Graph2D` for pass ordering

Pass contract uses:

- `definePass`
- `use`
- `input`
- `output`
- `require`
- `params`
- `priority`

## Material model

Materials are unified under `ShaderMaterial`.

- `kind: shader`
- `preset: standard | pbr`

`standard` and `pbr` are closed presets in this phase.

## Input model

Pointer input is global stream only. Legacy target/layer routed relay is removed.

## ID ownership rule

Host owns logical IDs and guarantees validity/uniqueness. Core owns physical handles, caches and runtime resources.

## Documentation sync rule

When changing composition, graph, or material contracts, update these docs in the same phase:

- `docs/REALM-ARCH.md`
- `docs/RENDER-GRAPH.md`
- `docs/GLOSSARY.md`
- impacted `docs/cmds/*`
