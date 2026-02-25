# CmdAudioStateGet

Returns current audio runtime state known by the core.

## Arguments

| Field           | Type | Description |
| --------------- | ---- | ----------- |
| includeListener | bool | Include current listener binding (default: `true`) |
| includeSources  | bool | Include tracked source states (default: `true`) |
| includeStreams  | bool | Include active stream progress states (default: `true`) |

## Response

Returns `CmdResultAudioStateGet`:

| Field    | Type                           | Description |
| -------- | ------------------------------ | ----------- |
| success  | bool                           | Whether retrieval succeeded |
| message  | String                         | Status or error message |
| listener | Option<AudioListenerBindingState> | Current listener binding |
| sources  | Vec<AudioSourceStateEntry>     | Tracked source states |
| streams  | Vec<AudioStreamStateEntry>     | Active resource stream states |

### AudioListenerBindingState

- `realmId`
- `modelId`

### AudioSourceStateEntry

- `sourceId`
- `realmId` (optional binding)
- `modelId` (optional binding)
- `position`, `velocity`, `orientation`
- `gain`, `pitch`

### AudioStreamStateEntry

- `resourceId`
- `receivedBytes`
- `totalBytes`
- `complete`
