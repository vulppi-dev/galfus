# CmdUiDocumentCreate

Creates a UI document with a realm reference.

## Arguments

| Field       | Type        | Description                    |
| ----------- | ----------- | ------------------------------ |
| documentId  | u32         | Logical UI document ID         |
| realmId     | u32         | Realm ID reference (late-bound) |
| rect        | Vec4        | Document rect (x, y, w, h)     |
| themeId     | Option<u32> | Optional theme ID              |

## Response

Returns `CmdResultUiDocumentCreate`:

| Field       | Type        | Description                   |
| ----------- | ----------- | ----------------------------- |
| success     | bool        | Whether the document was created |
| message     | String      | Status or error message       |
| documentId  | Option<u32> | ID of the created document    |

## Notes

- `realmId` is late-bound. Document creation is accepted even if the realm is not available yet.
