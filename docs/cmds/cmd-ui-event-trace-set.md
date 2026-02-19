# CmdUiEventTraceSet

Configura o trace de eventos de ponteiro no roteamento UI/realm/target.

## Arguments

| Field           | Type | Description |
| --------------- | ---- | ----------- |
| level           | Option<PointerTraceLevel> | Nível de trace (`off`, `errors`, `basic`, `full`) |
| samplingPercent | Option<u8> | Sampling de 0..100 |

## Response

Returns `CmdResultUiEventTraceSet`:

| Field           | Type | Description |
| --------------- | ---- | ----------- |
| success         | bool | Configuração aplicada |
| message         | String | Status ou erro |
| level           | Option<PointerTraceLevel> | Nível final ativo |
| samplingPercent | Option<u8> | Sampling final ativo |
