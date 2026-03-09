# CmdSystemBuildVersionGet

Retorna a versão de compilação do core.

## Arguments

Sem argumentos.

## Response

Retorna `CmdResultSystemBuildVersionGet`:

| Field | Type | Description |
| --- | --- | --- |
| success | bool | Se a leitura foi bem-sucedida |
| message | String | Mensagem de status |
| buildVersion | String | Versão de compilação (`CARGO_PKG_VERSION`) |
