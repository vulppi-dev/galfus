# CmdMaterialUpsert

Upserts a unified `ShaderMaterial` (`create` or `update`).

## Material model

Current contract is replace-only vNext:

- `kind`: `shader`
- `preset`: `standard` or `pbr`
- `options`: preset-specific payload (`standard` or `pbr`)

Legacy `kind=standard|pbr` payloads are invalid.

## Create payload (`CmdMaterialCreateArgs`)

Required:

- `materialId`
- `kind = "shader"`

Optional:

- `label`
- `preset` (defaults to `standard`)
- `options`

## Update payload (`CmdMaterialUpdateArgs`)

Required:

- `materialId`

Optional:

- `label`
- `kind` (`shader`)
- `preset`
- `options`

## Response

`{ success, message }`.

On command failure, runtime also emits `SystemEvent::Error` with `scope="command"`.
