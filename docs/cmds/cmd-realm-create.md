# CmdRealmCreate

Creates a new realm.

Realms represent renderable contexts (3D/2D). They can later be bound to
logical targets through `CmdTargetLayerUpsert` to form the auto-graph.

## Arguments

| Field         | Type           | Description |
| ------------- | -------------- | ----------- |
| kind          | RealmKind      | Realm kind (`three-d` or `two-d`) |
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

## Validation Rules

- `kind` must be a valid realm kind.
- Realm/target/window binding is resolved later by target commands (`CmdTargetUpsert` + `CmdTargetLayerUpsert`).

When validation fails:
- command response returns `success = false`
- host also receives `SystemEvent::Error` (`scope = "command"`).
