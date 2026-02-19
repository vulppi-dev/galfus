# CmdSystemDiagnosticsSet

Configura tracing de ponteiro e profiling em runtime.

## Arguments

Todos os campos são opcionais.

| Field | Type | Description |
| --- | --- | --- |
| profilingEnabled | bool | Habilita/coleta profiling |
| profilingDetail | `basic \| full` | Define granularidade de saída do profiling |
| profilingSamplingPercent | u8 | Sampling de profiling (`0..=100`) |
| profilingWindowFrames | u8 | Tamanho da janela móvel (`1..=120`) |
| traceLevel | `off \| errors \| basic \| full` | Nível de trace para eventos de ponteiro |
| traceSamplingPercent | u8 | Sampling de trace (`0..=100`) |

## Response

Retorna `CmdResultSystemDiagnosticsSet`:

| Field | Type | Description |
| --- | --- | --- |
| success | bool | Se a configuração foi aplicada |
| message | String | Mensagem de status |
