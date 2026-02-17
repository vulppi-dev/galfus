# CmdRealmDispose

Disposes a realm and removes its auto-graph links.

Dispose semantics:
- Removes the realm and direct dependencies:
  - target layers owned by the realm
  - generated auto-links (`surface/present/connector`)
  - direct connector captures tied to removed connectors
  - UI realm state/documents when realm kind is `two-d`
- If the disposed realm owned an output surface, the surface is removed and caches are pruned.
- Indirect dependencies continue via fallback where applicable.

## Arguments

| Field   | Type | Description |
| ------- | ---- | ----------- |
| realmId | u32  | ID of the realm to dispose |

## Response

Returns `CmdResultRealmDispose`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the realm was disposed |
| message | String | Status or error message        |
