# CmdWindowMeasurement

Aplica e consulta medições de janela em um único comando.

Agrupa:
- posição (`set/get`)
- tamanho interno (`set/get`)
- tamanho externo (`get`)
- tamanho de surface (`get`)

## Platform Notes

- **WASM:** usa a mesma estrutura de resposta do nativo.
  Enquanto a janela ainda não estiver pronta, retorna payload vazio com `success=true` (eventual consistency).

## Arguments

| Field          | Type         | Description |
| -------------- | ------------ | ----------- |
| windowId       | u32          | ID da janela |
| position       | Option<IVec2> | Novo outer position (opcional) |
| size           | Option<UVec2> | Novo inner size (opcional) |
| getPosition    | bool         | Inclui `position` na resposta |
| getSize        | bool         | Inclui `size` na resposta |
| getOuterSize   | bool         | Inclui `outerSize` na resposta |
| getSurfaceSize | bool         | Inclui `surfaceSize` na resposta |

## Response

Retorna `CmdResultWindowMeasurement`:

| Field       | Type          | Description |
| ----------- | ------------- | ----------- |
| success     | bool          | Se o comando foi aplicado com sucesso |
| message     | String        | Status (inclui estado transitório/deferred) |
| position    | Option<IVec2> | Posição atual/atualizada quando solicitada |
| size        | Option<UVec2> | Tamanho interno atual/atualizado quando solicitado |
| outerSize   | Option<UVec2> | Tamanho externo atual quando solicitado |
| surfaceSize | Option<UVec2> | Tamanho atual da surface quando solicitado |

## Notes

- Campos de patch são opcionais: envie apenas o que deseja alterar.
- `position` e `size`, quando enviados, também são retornados no payload de resposta.
- Se `windowId`/GPU ainda não estiverem prontos, o comando retorna `success=true` com campos de leitura vazios (eventual consistency).
