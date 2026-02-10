# CmdRealmCreate

Creates a new realm.

Realms represent renderable contexts (3D/2D). They can later be bound to
logical targets through `CmdTargetBindUpsert` to form the auto-graph.

## Arguments

| Field         | Type           | Description |
| ------------- | -------------- | ----------- |
| kind          | RealmKind      | Realm kind (`three-d` or `two-d`) |
| outputSurfaceId | Option<u32>  | (Optional) Explicit output surface ID (rare; usually omitted) |
| hostWindowId  | Option<u32>    | (Optional) Host window ID for default realm association |
| importance    | Option<u8>     | (Optional) Scheduling priority (default: 1) |
| cachePolicy   | Option<u8>     | (Optional) Cache policy (default: 0) |
| flags         | Option<u32>    | (Optional) Realm flags (reserved) |

## Response

Returns `CmdResultRealmCreate`:

| Field   | Type        | Description                   |
| ------- | ----------- | ----------------------------- |
| success | bool        | Whether the realm was created |
| message | String      | Status or error message       |
| realmId | Option<u32> | ID of the created realm       |
