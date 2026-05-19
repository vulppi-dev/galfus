# cmd-realm-get

## Purpose

Query or mutate core-managed state.

## Request

MessagePack command envelope with the command `type` and command-specific `content`.

## Response

- `success: bool`
- `message: string`
- command-specific DTO fields (no raw texture/audio buffers returned)

## Notes

- Host should treat core as source of truth and keep only IDs.
- Heavy payload bytes (image/audio raw buffers) are not returned by get/list.
