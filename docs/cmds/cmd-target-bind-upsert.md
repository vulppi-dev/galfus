# CmdTargetBindUpsert

Upserts a logical bind between a realm and a target.

## Arguments

| Field    | Type       | Description                     |
| -------- | ---------- | ------------------------------- |
| realmId  | u32        | Logical realm ID                |
| targetId | u64        | Logical target ID               |
| layout   | TargetBindLayout | Layout and routing configuration |

### TargetBindLayout

| Field      | Type        | Description                         |
| ---------- | ----------- | ----------------------------------- |
| rect       | Vec4        | Composition rectangle (x, y, w, h) |
| zIndex     | i32         | Layer order                         |
| blendMode  | u32         | Blend mode selector                 |
| clip       | Option<Vec4>| Optional clip rect                  |
| inputFlags | u32         | Input flags (bitmask)               |

## Response

Returns `CmdResultTargetBindUpsert`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the bind was upserted |
| message | String | Status or error message      |
