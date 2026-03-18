# CmdRenderGraphUpsert

Creates or updates a render graph resource in the global catalog.

Realms can later bind this graph via `CmdRealmRenderGraphBind` and reuse it across multiple realms.

## Arguments

| Field         | Type            | Description |
| ------------- | --------------- | ----------- |
| renderGraphId | u32             | Logical render graph ID (host-managed) |
| graph         | RenderGraphDesc | Graph definition (`graphId`, `nodes`, `edges`, `resources`) |

## Response

Returns `CmdResultRenderGraphUpsert`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Whether upsert succeeded |
| message | String | Status or error message |

## Validation Rules

- `renderGraphId` must not use core-reserved default IDs.
- `graph` must pass render-graph validation (`passId` known, resource refs declared, DAG order valid).

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "render-graph"`).
