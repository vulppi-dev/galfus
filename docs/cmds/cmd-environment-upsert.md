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
- profiles can be bound per target layer via `CmdTargetLayerUpsert.environmentId`.

`EnvironmentConfig` fields:

- `msaa { enabled, sampleCount }`
- `skybox { mode, intensity, rotation, groundColor, horizonColor, skyColor, cubemapTextureId? }`
- `clearColor` (`Vec4`, RGBA)
- `post` (post-processing block: `filter_*`, outline, SSAO, bloom)

## Response

`{ success, message }`
