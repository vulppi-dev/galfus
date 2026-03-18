# CmdRealmRenderGraphBind

Binds a realm to a render graph resource by logical ID.

## Arguments

| Field         | Type | Description |
| ------------- | ---- | ----------- |
| realmId       | u32  | Realm to bind |
| renderGraphId | u32  | Render graph resource ID |

## Response

Returns `CmdResultRealmRenderGraphBind`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Whether bind succeeded |
| message | String | Status or error message |

## Validation Rules

- `realmId` must exist.
- `renderGraphId` must exist in the render graph catalog.
- `two-d` realms can bind only UI-only graphs (`passId = "ui"` for all nodes).

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "render-graph"`).
