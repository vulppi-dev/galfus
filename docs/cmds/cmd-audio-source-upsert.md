# CmdAudioSourceUpsert

Upserts an audio source (`Create` or `Update`).

## Arguments

Accepts one of:

- `CmdAudioSourceCreateArgs`
- `CmdAudioSourceUpdateArgs`

Key fields include:

- `sourceId` (required)
- `resourceId` (audio resource binding)
- `mode`, `gain`, `pitch`, `intensity`
- optional spatial binding to model/listener context

## Notes

- If the runtime audio backend is unavailable, commands fail deterministically.
- Decode/stream readiness is asynchronous and reported via `SystemEvent::AudioReady` and `SystemEvent::AudioStreamProgress`.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
