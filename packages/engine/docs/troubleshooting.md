# Troubleshooting

## Engine already initialized

`initEngine()` can only be called once per lifetime. If you need to restart, call `disposeEngine()` first.

## No rendering or input updates

Make sure `tick()` is called exactly once per frame with a monotonic timestamp and delta in milliseconds.

## MessagePack serialization errors

If you see `galfusSendQueue failed`, the host/core command schema might be out of sync. Ensure all packages are built from compatible versions.
The most common causes are stale TypeScript command shapes, invalid nested UI/audio payloads, or render graph payloads that do not match the current engine/core contract.

## MSAA errors

If MSAA fails to initialize, your adapter likely doesn't support the requested sample count for the format used by the engine (e.g. `rgba16float` + `depth32float`). Use `1` or probe for supported counts.
