# CmdMaterialUpsert

Upserts a unified `ShaderMaterial` instance (`create` or `update`).

## Material model

Current contract in this phase:

- `kind`: `shader`
- `slug`: material definition reference (`standard`, `pbr`, or custom definition slug)
- `options`: definition-specific payload

Definitions are managed through material definition commands and compiled by the unified composer.

## Create payload (`CmdMaterialCreateArgs`)

Required:

- `materialId`
- `kind = "shader"`

Optional:

- `label`
- `slug` (defaults to `standard`)
- `options`

## Update payload (`CmdMaterialUpdateArgs`)

Required:

- `materialId`

Optional:

- `label`
- `kind` (`shader`)
- `slug`
- `options`

## Response

`{ success, message }`.

On command failure, runtime also emits `SystemEvent::Error` with `scope="command"`.
