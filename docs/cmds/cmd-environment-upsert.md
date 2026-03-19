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
- `skybox { mode, intensity, rotation, groundColor, horizonColor, skyColor, horizonGroundThreshold, horizonSkyThreshold, directionalLights[{ lightId, solidSize, gradientSize }], cubemapTextureId? }`
- `clearColor` (`Vec4`, RGBA)
- `post` (post-processing block: `filter_*`, outline, SSAO, bloom)

Procedural skybox notes:
- horizon center is fixed at 90deg (half-sphere split);
- `horizonGroundThreshold` and `horizonSkyThreshold` control influence toward ground/sky (`0.0` favors horizon color, `1.0` favors ground/sky color with thinner transition);
- sun sizing uses normalized hemisphere scale:
  - `solidSize`: `0.0` none, `1.0` hemisphere, `2.0` full sky;
  - `gradientSize`: halo size in the same scale (typically >= `solidSize`).

## Response

`{ success, message }`
