# CmdRenderGraphList

Lists render graph resources currently registered in the global catalog.

## Arguments

No fields.

## Response

Returns `CmdResultRenderGraphList`:

| Field        | Type                    | Description |
| ------------ | ----------------------- | ----------- |
| success      | bool                    | Whether listing succeeded |
| message      | String                  | Status message |
| renderGraphs | Vec<RenderGraphEntry>   | Registered render graphs |

`RenderGraphEntry`:

| Field         | Type        | Description |
| ------------- | ----------- | ----------- |
| renderGraphId | u32         | Logical render graph ID |
| descHash      | u64         | Hash of the graph description used for compiled-plan cache |
| passCount     | usize       | Number of pass nodes in the plan |
| passIds       | Vec<String> | Ordered pass IDs from active plan |
