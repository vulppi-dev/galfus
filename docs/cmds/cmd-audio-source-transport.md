# CmdAudioSourceTransport

Applies transport actions to an audio source.

This consolidates:
- former `CmdAudioSourcePlay`
- former `CmdAudioSourcePause`
- former `CmdAudioSourceStop`

## Arguments

| Field      | Type                       | Description |
| ---------- | -------------------------- | ----------- |
| sourceId   | u32                        | Source ID |
| action     | AudioSourceTransportAction | `play`, `pause`, or `stop` |
| resourceId | Option<u32>                | Required for `play` |
| timelineId | Option<u32>                | Timeline layer (`play` default: `0`; `pause/stop`: all when omitted) |
| intensity  | Option<f32>                | Extra volume multiplier for `play` (default: `1.0`, clamped to `0..1`) |
| delayMs    | Option<u32>                | Optional delay for `play` |
| mode       | Option<AudioPlayMode>      | `once` or `loop` for `play` (default: `once`) |

## Response

Returns `CmdResultAudioSourceTransport`:

| Field   | Type   | Description |
| ------- | ------ | ----------- |
| success | bool   | Whether the transport action succeeded |
| message | String | Status or error message |
