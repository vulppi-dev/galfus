# CmdEnvironmentUpsert

Upserts an environment profile (create or update).

## Arguments

`CmdEnvironmentUpsert` accepts one of:

- `Create { environmentId, config }`
- `Update { environmentId, config }`

| Field         | Type              | Description |
| ------------- | ----------------- | ----------- |
| environmentId | u32               | Logical profile ID |
| config        | EnvironmentConfig | Profile payload |

Behavior:
- the profile is stored in the core environment pool;
- this profile becomes the current default fallback environment.

## Response

`{ success, message }`
