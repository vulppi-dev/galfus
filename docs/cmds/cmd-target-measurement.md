# CmdTargetMeasurement

Consulta medidas efetivas de um target lógico.

## Arguments

| Field         | Type | Description |
| ------------- | ---- | ----------- |
| targetId      | u64  | ID lógico do target |
| getSize       | bool | Inclui `size` e `sourceKind` na resposta |
| getWindowSize | bool | Inclui `windowSize` na resposta (quando aplicável) |

## Response

Retorna `CmdResultTargetMeasurement`:

| Field      | Type          | Description |
| ---------- | ------------- | ----------- |
| success    | bool          | Se o comando foi aplicado com sucesso |
| message    | String        | Status do comando |
| size       | Option<UVec2> | Tamanho efetivo do target |
| windowSize | Option<UVec2> | Tamanho da janela associada ao target (quando existir) |
| sourceKind | Option<String> | Origem da medida de `size`: `"surface"`, `"window-surface"` ou `"declared"` |

## Notes

- Se `targetId` ainda não estiver pronto, o comando retorna `success=true` com payload vazio.
- `size` prioriza medida real observável de `surface` (autolink/present window) antes de fallback declarado.
- Para targets ligados a janela em browser, a medida de surface considera HiDPI quando o host usa CSS sizing.
