# Galfus Internal API

This document describes the active architectural contracts.

## Runtime integration root

`galfus-runtime` remains the integration root for:

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

- `galfus-render`: render policy and pass/frame execution policy
- `galfus-realm-core`: composition DTO/state semantics shared by runtime
- `galfus-runtime`: applies commands and drives frame loop

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

Materials are unified under `ShaderMaterial` with `MaterialDefinition + MaterialInstance`.

- definition owns: `shaderType`, `shaderSource`, `shaderParamsSchema`, `basePreset`
- instance owns: `slug` (definition lookup) + per-material options/values
- closed builtins in this phase: `standard` and `pbr` (bootstrapped definitions)

Shader assembly is centralized:

- `generated_common_prelude` (shared contract/bindings/helpers)
- `source` (preset/custom logic)
- `generated_postlude` (entrypoint bridge)

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
