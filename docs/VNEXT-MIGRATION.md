# vNext Host Migration Guide

This guide describes the required host-side migration for vNext.

## 1. Targets and layers

Update target creation:

- keep only `kind = window | texture`
- for `window`, provide `windowId`
- for `texture`, provide optional `size`

Keep composition through `CmdTargetLayerUpsert` only.

## 2. Remove legacy composition assumptions

Delete host-side usage of:

- widget viewport targets
- realm planes
- connector/present/surface public pipeline assumptions

Treat those as internal runtime/render concerns.

## 3. Pointer input migration

Remove host code that expects target-routed pointer metadata/relay.

Consume pointer as global event stream.

## 4. Material payload migration

Replace old material payloads with unified shader material payload:

- `kind: "shader"`
- `preset: "standard" | "pbr"`

Keep preset-specific options under `options`.

## 5. Graph usage migration

Keep render graph resources bound by realm and use pass contracts with:

- `input`
- `output`
- `require`
- `priority`
- `params`

Rely on deterministic graph ordering by dependencies + priority.

## 6. Validation checklist

1. Run command payload serialization tests on the host side.
2. Smoke test a multi-target texture pipeline (`Scene -> Post -> Window`).
3. Confirm no runtime references remain to removed legacy commands/types.
4. Confirm material creation uses `kind: shader` only.

## 7. Rollout recommendation

1. deploy migration branch in staging
2. compare frame behavior and command errors against previous branch
3. release with vNext tag only after host payload compatibility is confirmed
