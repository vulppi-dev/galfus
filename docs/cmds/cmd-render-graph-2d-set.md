# CmdRenderGraph2DSet

Sets a host-defined render graph for a 2D realm. The core validates the graph and compiles an execution plan. If invalid and `fallback=true`, the core uses the default fallback graph.

## Arguments

| Field   | Type            | Description          |
| ------- | --------------- | -------------------- |
| realmId | u32             | ID of the 2D realm   |
| graph   | RenderGraphDesc | Graph description    |

## Response

Returns `CmdResultRenderGraph2DSet`:

| Field        | Type   | Description                            |
| ------------ | ------ | -------------------------------------- |
| success      | bool   | Whether the graph was accepted         |
| fallbackUsed | bool   | Whether the fallback graph was applied |
| message      | String | Status or error message                |
