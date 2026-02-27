# CmdGeometryDispose

Removes a geometry resource.

## Arguments

| Field      | Type | Description                  |
| ---------- | ---- | ---------------------------- |
| geometryId | u32  | ID of the geometry to remove |

## Response

Returns `CmdResultGeometryDispose`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the geometry was removed |
| message | String | Status or error message          |
