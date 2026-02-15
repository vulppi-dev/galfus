# CmdTargetUpsert

Upserts a logical target used by the auto-graph system.

## Arguments

| Field         | Type                  | Description                                   |
| ------------- | --------------------- | --------------------------------------------- |
| targetId      | u64                   | Logical target ID                             |
| kind          | TargetKind            | "window", "ui-plane", "texture" |
| windowId | Option<u32>           | Required for `window`, `ui-plane` |
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

## Validation Rules

- `windowId` is required for `kind = "window"` and `"ui-plane"`.
- `windowId` must reference an existing window.
- `windowId` is not accepted for `kind = "texture"`.
- `size` is accepted only for `kind = "texture"`.

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "command"`).
