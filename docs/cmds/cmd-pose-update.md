# CmdPoseUpdate

Updates skinning pose matrices for a model.

This uploads a bone matrix palette from an upload buffer and applies it to the
model's skinning allocation. The buffer is consumed and discarded after a
successful update.

## Arguments

| Field            | Type | Description |
| ---------------- | ---- | ----------- |
| windowId         | u32  | ID of the window |
| modelId          | u32  | ID of the model to update |
| boneCount        | u32  | Number of bones in the palette (must be > 0 and within engine limits) |
| matricesBufferId | u64  | Upload buffer ID containing `boneCount` Mat4 values |

## Response

Returns `CmdResultPoseUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the pose was updated  |
| message | String | Status or error message       |
