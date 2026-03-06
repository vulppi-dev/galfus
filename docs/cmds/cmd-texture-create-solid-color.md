# CmdTextureCreateSolidColor

Creates a 1x1 texture with a solid color.

## Arguments

| Field        | Type                        | Description                                                        |
| ------------ | --------------------------- | ------------------------------------------------------------------ |
| textureId    | u32                         | Unique ID for the texture                                          |
| label        | Option<String>              | (Optional) Semantic name                                           |
| color        | Vec4                        | Color in RGBA                                                      |
| srgb         | Option<bool>                | (Optional) Use sRGB (default: true)                                |
| mode         | TextureCreateMode           | (Optional) "standalone" or "forward-atlas" (default: "standalone") |
| atlasOptions | Option<ForwardAtlasOptions> | (Optional) Options for atlas allocation                            |

### ForwardAtlasOptions

- **tilePx**: u32 (tile size in pixels, default: 256)
- **layers**: u32 (atlas layers, default: 1)

## Response

Returns `CmdResultTextureCreateSolidColor`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was created |
| message | String | Status or error message         |

## Runtime Behavior

- If GPU device/queue is not ready yet, creation is deferred and replayed automatically.
- Command order is not required; texture creation converges when dependencies become available.
