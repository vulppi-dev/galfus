# CmdAudioListenerDispose

Disposes the current listener binding for the given window.

## Arguments

| Field    | Type | Description            |
| -------- | ---- | ---------------------- |
| realmId  | u32  | Realm owning listener  |

## Response

Returns `CmdResultAudioListenerDispose`:

| Field   | Type   | Description               |
| ------- | ------ | ------------------------- |
| success | bool   | Whether dispose succeeded |
| message | String | Status or error message   |
