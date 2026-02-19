# CmdUiThemeDispose

Disposes a UI theme resource.

## Arguments

| Field   | Type | Description         |
| ------- | ---- | ------------------- |
| themeId | u32  | Logical UI theme ID |

## Response

Returns `CmdResultUiThemeDispose`:

| Field   | Type | Description                  |
| ------- | ---- | ---------------------------- |
| success | bool | Whether the theme was disposed |
| message | String | Status or error message   |
