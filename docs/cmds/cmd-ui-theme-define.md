# CmdUiThemeDefine

Defines or updates a UI theme resource.

## Arguments

| Field   | Type              | Description                          |
| ------- | ----------------- | ------------------------------------ |
| themeId | u32               | Logical UI theme ID                  |
| version | Option<u32>       | Optional version (auto-bumps if none) |
| data    | Map<String, Value> | Tokens de tema                       |
| fontData | Map<String, bytes> | Blob de fontes por nome             |
| fontFamilies | Map<String, Vec<String>> | Famílias de fontes (ordem de fallback) |

## Response

Returns `CmdResultUiThemeDefine`:

| Field    | Type        | Description                       |
| -------- | ----------- | --------------------------------- |
| success  | bool        | Whether the theme was stored     |
| message  | String      | Status or error message          |
| themeId  | Option<u32> | ID of the theme                  |
| version  | Option<u32> | Stored version                   |
