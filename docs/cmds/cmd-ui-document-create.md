# CmdUiDocumentCreate

Creates a UI document bound to a realm.

## Arguments

| Field       | Type        | Description                    |
| ----------- | ----------- | ------------------------------ |
| documentId  | u32         | Logical UI document ID         |
| realmId     | u32         | Realm ID that owns the document |
| rect        | Vec4        | Document rect (x, y, w, h)     |
| themeId     | Option<u32> | Optional theme ID              |

## Response

Returns `CmdResultUiDocumentCreate`:

| Field       | Type        | Description                   |
| ----------- | ----------- | ----------------------------- |
| success     | bool        | Whether the document was created |
| message     | String      | Status or error message       |
| documentId  | Option<u32> | ID of the created document    |
