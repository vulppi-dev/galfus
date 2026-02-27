# CmdPoseUpdate

Updates skinning pose matrices for a model.

This uploads a bone matrix palette from an upload buffer and applies it to the
model's skinning allocation. The buffer is consumed and discarded after a
successful update.

## Arguments

| Field            | Type | Description |
| ---------------- | ---- | ----------- |
| realmId          | u32  | ID of the realm owning the model |
| modelId          | u32  | ID of the model to update |
| boneCount        | u32  | Number of bones in the palette (must be > 0 and within engine limits) |
| matricesBufferId | u64  | Upload buffer ID containing `boneCount` Mat4 values |
| windowId         | Option<u32> | (Optional) Window hint for pose upload path |

## Response

Returns `CmdResultPoseUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the pose was updated  |
| message | String | Status or error message       |
