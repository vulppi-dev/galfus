# CmdRealmDispose

Disposes a realm and removes its auto-graph links.

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
