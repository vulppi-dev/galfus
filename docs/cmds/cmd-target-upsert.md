# CmdTargetUpsert

Upserts a logical target used by the auto-graph system.

## Arguments

| Field         | Type                  | Description                                   |
| ------------- | --------------------- | --------------------------------------------- |
| targetId      | u64                   | Logical target ID                             |
| kind          | TargetKind            | "window", "widget-realm-viewport", "realm-plane", "texture" |
| windowId | Option<u32>           | Required only for `window` |
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

- `windowId` is required for `kind = "window"`.
- `windowId` is not accepted for `kind = "texture"`.
- `widget-realm-viewport` is intended for `UiNodeProps::WidgetRealmViewport` sampling.
  It participates in auto-graph routing/input, but is not composed as a visible connector overlay on the host UI realm.
- `realm-plane` is rendered as double-sided in rasterization.
  Pointer raycast accepts front-face hits only.
- `size` is accepted only for `kind = "texture"`.

## Notes

- `windowId` is treated as a late-bound reference.
  Target upsert is accepted even if the window does not exist yet.

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "command"`).
