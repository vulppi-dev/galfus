# Rendering and Environment

## Environment

Use `configureEnvironment()` to control skybox and MSAA.

```ts
configureEnvironment(WINDOW_ID, {
  msaa: { enabled: true, sampleCount: 4 },
  skybox: {
    mode: 'procedural',
    intensity: 1.0,
    rotation: 0,
    groundColor: [0.02, 0.03, 0.04],
    horizonColor: [0.12, 0.16, 0.22],
    skyColor: [0.2, 0.35, 0.6],
    horizonGroundThreshold: 0.45,
    horizonSkyThreshold: 0.55,
    directionalLights: [
      { lightId: 1, solidSize: 0.0018, gradientSize: 0.0287 },
    ],
    cubemapTextureId: null,
  },
  clearColor: [0, 0, 0, 0],
  post: {
    filterEnabled: true,
    filterExposure: 1,
    filterGamma: 1,
    filterSaturation: 1,
    filterContrast: 1,
    filterVignette: 0,
    filterGrain: 0,
    filterChromaticAberration: 0,
    filterBlur: 0,
    filterSharpen: 0,
    filterTonemapMode: 0,
    outlineEnabled: false,
    outlineStrength: 0,
    outlineThreshold: 0.2,
    outlineWidth: 1,
    outlineQuality: 1,
    filterPosterizeSteps: 0,
    cellShading: false,
    ssaoEnabled: false,
    ssaoStrength: 1,
    ssaoRadius: 0.75,
    ssaoBias: 0.025,
    ssaoPower: 1.5,
    ssaoBlurRadius: 2,
    ssaoBlurDepthThreshold: 0.02,
    bloomEnabled: false,
    bloomThreshold: 1,
    bloomKnee: 0.5,
    bloomIntensity: 0.8,
    bloomScatter: 0.7,
  },
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
  tileResolution: 1024,
  atlasTilesW: 8,
  atlasTilesH: 8,
  atlasLayers: 2,
  virtualGridSize: 1,
  smoothing: 2,
  normalBias: 0.01,
});
```
