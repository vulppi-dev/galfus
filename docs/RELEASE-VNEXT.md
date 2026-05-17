# vNext Release Notes (Draft)

Release tag suggestion: `vnext.0.1.0`

## Summary

vNext replaces the legacy composition architecture with a reduced core model based on:

- `Realm`
- `Target`
- `Layer`
- `Texture`
- `FrameGraph`
- `RenderInvocation`
- `Graph3D` / `Graph2D`

## Key changes

1. Public composition simplification:
- targets limited to `window|texture`
- layers as single composition primitive

2. Internal scheduling model:
- global `FrameGraph` ordering by texture dependencies
- deterministic target execution ordering

3. Pass/graph model:
- split `Graph3D` and `Graph2D`
- pass contracts with `input/output/require/priority/params`

4. Material model:
- unified `ShaderMaterial`
- closed presets `standard|pbr`

5. Legacy stack removals:
- legacy UI/raycast pipeline
- target-routed pointer relay
- deprecated composition abstractions

## Breaking changes

See:

- [VNEXT-BREAKING-CHANGES.md](VNEXT-BREAKING-CHANGES.md)
- [VNEXT-MIGRATION.md](VNEXT-MIGRATION.md)

## Validation snapshot

Core validation executed during final phases:

- `cargo test -p vulfram-realm-core`
- `cargo test -p vulfram-render`
- `cargo test -p vulfram-runtime`
- `cargo test -p vulfram-demo`

All passing in final phase validation run.

## Publish checklist

1. confirm changelog label coverage on merged PRs
2. publish release artifacts through CI workflow
3. attach this note body (or generated equivalent) to release tag
4. verify host migration docs are linked in release description
