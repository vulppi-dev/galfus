# CmdEnvironmentDispose

Disposes an environment profile from the core pool.

## Arguments

| Field         | Type | Description |
| ------------- | ---- | ----------- |
| environmentId | u32  | Logical profile ID |

## Response

Returns `CmdResultEnvironment`:

| Field   | Type   | Description                         |
| ------- | ------ | ----------------------------------- |
| success | bool   | Whether the environment was disposed |
| message | String | Status or error message              |

Notes:
- if the disposed profile was the default fallback, the core picks another profile deterministically (lowest ID), or none when pool is empty;
- all `TargetLayer` entries referencing this `environmentId` are reset to `null`.
