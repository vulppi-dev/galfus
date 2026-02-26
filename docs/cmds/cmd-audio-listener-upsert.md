# CmdAudioListenerUpsert

Upserts an audio listener (`Create` or `Update`).

## Arguments

Accepts one of:

- `CmdAudioListenerCreateArgs`
- `CmdAudioListenerUpdateArgs`

Key fields:

- Create: `realmId`, `modelId`
- Update: `position`, `velocity`, `forward`, `up`

## Notes

- Listener and source bindings are resolved every tick by the core audio proxy.
- If listener and source are bound to the same model, spatialization is bypassed.

## Response

Returns `{ success, message }`.

On failure, the core also emits `SystemEvent::Error` (`scope="command"`).
