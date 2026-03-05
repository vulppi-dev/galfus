# CmdAudioSourceUpsert

Upserts an audio source (`Create` or `Patch`).

## Arguments

Accepts one of:

- `CmdAudioSourceCreateArgs`
- `CmdAudioSourceUpdateArgs`

Key fields include:

- `sourceId` (required)
- Create: `realmId`, `modelId`, `position`, `velocity`, `orientation`, `gain`, `pitch`, `spatial`
- Patch: optional fields for `realmId`, `modelId`, `position`, `velocity`, `orientation`, `gain`, `pitch`, `spatial`

## Notes

- If the runtime audio backend is unavailable, commands fail deterministically.
- `resourceId` playback binding is configured by `CmdAudioSourceTransport` (`action = "play"`).
- Decode/stream readiness is asynchronous and reported via `SystemEvent::AudioReady` and `SystemEvent::AudioStreamProgress`.
- `realmId`/`modelId` references are late-bound and can be resolved after upsert.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
