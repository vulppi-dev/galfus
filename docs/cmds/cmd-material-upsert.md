# CmdMaterialUpsert

Upserts a unified `ShaderMaterial` instance (`create` or `update`).

## Material model

Current contract in this phase:

- `kind`: `shader`
- `slug`: material definition reference (`standard`, `pbr`, or custom definition slug)
- `options`: definition-specific payload

Definitions are managed through material definition commands and compiled by the unified composer.
`realm-2d` and `realm-3d` share the same definition/instance workflow.

## Create payload (`CmdMaterialCreateArgs`)

Required:

- `materialId`
- `kind = "shader"`

Optional:

- `label`
- `slug` (defaults to `standard`)
- `options`

Restrictions:

- `materialId` must not be in core-reserved range (`4294901761..=4294967295` for `u32`).

## Update payload (`CmdMaterialUpdateArgs`)

Required:

- `materialId`

Optional:

- `label`
- `kind` (`shader`)
- `slug`
- `options`

Restrictions:

- Reserved core material IDs cannot be updated/disposed by host commands.

## Response

`{ success, message }`.

On command failure, runtime also emits `SystemEvent::Error` with `scope="command"`.
