# Rendering and Environment

## Environment

Use `configureEnvironment()` to control skybox and MSAA.
The payload is sparse: omit anything you do not want to override.
If a value already matches the core default, the engine also avoids serializing it.

```ts
configureEnvironment(WINDOW_ID, {
  msaa: { sampleCount: 4 },
  skybox: {
    mode: 'procedural',
    groundColor: [0.02, 0.03, 0.04],
    horizonColor: [0.12, 0.16, 0.22],
    skyColor: [0.2, 0.35, 0.6],
    directionalLights: [{ lightId: 1, solidSize: 0.0018, gradientSize: 0.0287 }]
  },
  post: {
    outlineEnabled: false,
    outlineThreshold: 0.2
  }
});
```

### Skybox Notes

- `horizonGroundThreshold` and `horizonSkyThreshold` control horizon blending.
- `directionalLights` maps procedural sun discs to directional light IDs.
- `cubemapTextureId` is used when `mode: 'cubemap'`.

### MSAA Notes

MSAA support depends on the adapter and texture formats. In WebGPU, sample counts often support only `1` or `4`. If a count is unsupported, the core will fail to create the render target.

## Shadows

Configure shadows via `configureShadows()`:

```ts
configureShadows(WINDOW_ID, {
  tileResolution: 1024
});
```
