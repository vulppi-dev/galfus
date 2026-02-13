# CmdTargetUpsert

Upserts a logical target used by the auto-graph system.

## Arguments

| Field         | Type                  | Description                                   |
| ------------- | --------------------- | --------------------------------------------- |
| targetId      | u64                   | Logical target ID                             |
| kind          | TargetKind            | "window", "realm-viewport", "ui-plane", "texture" |
| windowId | Option<u32>           | Required for `window`, `realm-viewport`, `ui-plane` |
| size  | Option<UVec2>         | Valid only for `texture` targets |
| formatPolicy  | Option<SurfaceFormat> | Optional color/depth format policy            |
| alphaPolicy   | Option<AlphaMode>     | Optional alpha policy                         |
| msaaSamples   | Option<u32>           | Optional MSAA samples                         |

## Response

Returns `CmdResultTargetUpsert`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the target was upserted |
| message | String | Status or error message         |
