# CmdAudioResourceUpsert

Upserts audio resource bytes (single-shot or streaming chunks).

This consolidates:
- former `CmdAudioResourceCreate`
- former `CmdAudioResourcePush`

## Arguments

| Field       | Type        | Description |
| ----------- | ----------- | ----------- |
| resourceId  | u32         | Logical ID for the audio asset |
| bufferId    | u64         | Upload buffer ID containing data (`BinaryAsset`) |
| totalBytes  | Option<u64> | Total stream size. When present, starts/continues stream mode |
| offsetBytes | Option<u64> | Chunk offset in bytes (default: `0`) |

## Response

Returns `CmdResultAudioResourceUpsert`:

| Field         | Type   | Description |
| ------------- | ------ | ----------- |
| success       | bool   | Whether the upsert was accepted |
| message       | String | Status or error message |
| pending       | bool   | Whether decode is pending asynchronously |
| receivedBytes | u64    | Total bytes received in stream mode |
| totalBytes    | u64    | Expected total bytes in stream mode |
| complete      | bool   | Whether stream mode has received all bytes |

## Notes

- For one-shot uploads (`totalBytes = null` and no existing stream), bytes are queued directly for decode.
- In stream mode, progress is emitted through `SystemEvent::AudioStreamProgress`.
- Decode completion is emitted through `SystemEvent::AudioReady`.
