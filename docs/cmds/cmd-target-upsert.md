# CmdTargetUpsert

Upserts a logical target used by the auto-graph system.

## Arguments

| Field         | Type                  | Description                                   |
| ------------- | --------------------- | --------------------------------------------- |
| targetId      | u64                   | Logical target ID                             |
| kind          | TargetKind            | "window", "viewport-embed", "panel-embed", "texture" |
| ownerWindowId | Option<u32>           | Owning window (required for window/embeds)    |
| sizeOverride  | Option<UVec2>         | Optional size override for the target         |
| formatPolicy  | Option<SurfaceFormat> | Optional color/depth format policy            |
| alphaPolicy   | Option<AlphaMode>     | Optional alpha policy                         |
| msaaSamples   | Option<u32>           | Optional MSAA samples                         |

## Response

Returns `CmdResultTargetUpsert`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the target was upserted |
| message | String | Status or error message         |
