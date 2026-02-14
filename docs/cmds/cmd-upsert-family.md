# Resource Upsert Commands

The command surface now includes unified upsert variants for common create/update pairs:

- `CmdCameraUpsert`
- `CmdModelUpsert`
- `CmdLightUpsert`
- `CmdMaterialUpsert`
- `CmdGeometryUpsert`
- `CmdEnvironmentUpsert`
- `CmdAudioListenerUpsert`
- `CmdAudioSourceUpsert`

Legacy `Create`/`Update` command pairs for these resources were removed from
`EngineCmd`. Use only the upsert variants.

Each upsert command accepts either create-shape arguments or update-shape arguments
for the same resource.

Response is standardized as:

```json
{ "success": true|false, "message": "..." }
```

On `success=false`, the host also receives `SystemEvent::Error` with
`scope="command"`.
