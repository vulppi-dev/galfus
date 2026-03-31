# Plano de refatoração do Vulfram para workspace multi-crates

## Objetivo

Refatorar o **Vulfram** para um **workspace Cargo** com múltiplos crates, reconstruindo a engine por camadas bem isoladas, com contratos explícitos entre subsistemas e sem contaminar responsabilidades entre crates.

A estratégia não será uma extração conservadora do monólito atual. Vamos usar o código existente como referência arquitetural e funcional, mas a nova estrutura será montada de baixo para cima, começando pelos crates folha e evoluindo por incrementos testáveis até a integração completa.

O critério principal não é preservar a organização atual durante a migração; é chegar ao final com uma arquitetura limpa, com fronteiras reais e verificáveis.

---

## Princípios da refatoração

### Princípios obrigatórios

- cada crate deve ter uma responsabilidade única e explícita
- crates baixos não dependem de crates altos
- contratos devem nascer antes das implementações pesadas
- tipos serializáveis e semânticos não devem depender de backend concreto
- nenhum crate deve conhecer detalhes internos de outro sem passar por tipos ou interfaces públicas
- toda fase deve deixar artefatos compiláveis e com testes automatizados proporcionais ao que foi introduzido
- a migração deve reduzir acoplamento estrutural, não apenas redistribuir arquivos

### O que vamos evitar

- copiar o monólito para vários crates e "arrumar depois"
- mover tipos para um crate novo mantendo dependências indevidas escondidas
- criar crates vazios sem contrato real, exceto quando houver justificativa arquitetural clara
- fazer `runtime` voltar a concentrar estado sem critério
- deixar `render` definir semântica de cena
- deixar `protocol` carregar tipos concretos de `wgpu`, `winit`, `web-sys` ou bindings

---

## Diagnóstico do estado atual

Hoje o projeto ainda está centralizado em um pacote único, com acoplamento estrutural relevante entre:

- runtime
- protocolo
- renderização
- janela/plataforma
- input
- áudio
- UI
- composição por realms/targets/surfaces
- demo/app local
- bindings de host

### Sintomas concretos observados

- `EngineState` concentra estado demais
- `UniversalState` mistura composição, UI, input routing, render graphs e recursos globais
- bindings vivem no mesmo crate da engine
- demo local está acoplada ao fluxo principal do runtime
- render ainda executa lógica acima da fronteira gráfica
- a separação entre cena semântica e backend gráfico ainda não é dura

### Leitura arquitetural do estado atual

O código atual serve como referência de comportamento e cobertura funcional, mas não como estrutura ideal para ser apenas "movida" para o workspace. A nova arquitetura será montada por reconstrução incremental, aproveitando conceitos e partes reutilizáveis, mas sem obrigação de reproduzir a mesma divisão interna do monólito.

---

## Estrutura alvo do workspace

```text
vulfram/
├── Cargo.toml
├── crates/
│   ├── vulfram-types/
│   ├── vulfram-protocol/
│   ├── vulfram-realm-core/
│   ├── vulfram-input/
│   ├── vulfram-realm-ui/
│   ├── vulfram-render/
│   ├── vulfram-audio/
│   ├── vulfram-runtime/
│   ├── vulfram-platform/
│   ├── vulfram-realm-3d/
│   ├── vulfram-realm-2d/
│   ├── vulfram-bindings-c/
│   ├── vulfram-bindings-wasm/
│   ├── vulfram-bindings-napi/
│   ├── vulfram-bindings-python/
│   ├── vulfram-bindings-lua/
│   └── vulfram-demo/
├── assets/
├── docs/
├── scripts/
└── examples/
```

A ordem acima nao representa a ordem de implementação. A implementação seguirá o grau de isolamento e maturidade de contrato, não a ordem alfabética.

---

## Responsabilidade de cada crate

### `vulfram-types`

Responsabilidade:
- IDs lógicos
- newtypes
- enums genéricos
- tipos base compartilhados
- descritores neutros
- structs utilitárias sem backend

Não deve conter:
- `wgpu`
- `winit`
- `web-sys`
- `napi`
- `pyo3`
- `mlua`
- lógica de runtime
- tipos de serialização amarrados ao protocolo se isso contaminar a base semântica

### `vulfram-protocol`

Responsabilidade:
- comandos
- eventos
- responses
- envelopes
- payloads serializáveis
- codec MessagePack
- contratos host-runtime

Não deve conter:
- lógica de runtime
- lógica de render
- regras de cena
- APIs de host
- `wgpu`
- `winit`
- `web-sys`

### `vulfram-realm-core`

Responsabilidade:
- composição semântica de realms, surfaces e targets
- target layers
- conectores e presents
- índices e caches semânticos
- regras determinísticas de resolução composicional
- base comum para 2D, 3D e UI quando realmente compartilhada

Não deve conter:
- pipelines de GPU
- shaders
- `wgpu`
- estado global do runtime
- dependência de `render`

### `vulfram-input`

Responsabilidade:
- eventos normalizados de input
- estado de teclado, mouse, touch, pointer e gamepad
- foco e captura em nível genérico
- utilitários e máquinas de estado de input

Não deve conter:
- DOM
- `winit`
- `web-sys`
- canvas ou janela concretos
- orchestration de runtime

### `vulfram-realm-ui`

Responsabilidade:
- documentos e árvores de UI
- estado de interação
- foco, hover, captura e navegação
- layout
- bridge entre input normalizado e semântica de UI

Não deve conter:
- backend gráfico concreto
- conhecimento do runtime global
- dependência direta de plataforma concreta

### `vulfram-render`

Responsabilidade:
- integração com `wgpu`
- device, queue, surfaces e recursos GPU
- pipelines, passes e shaders
- render graphs executáveis
- sincronização de dados semânticos para estruturas renderizáveis
- composição final de frame

Não deve conter:
- definição semântica da cena
- ownership da cena
- lógica de protocolo
- lógica de host bindings

### `vulfram-audio`

Responsabilidade:
- listener, sources e buffers
- backends desktop/web
- atualização e controle de reprodução
- espacialização e estado de áudio

Não deve conter:
- render
- input
- janela
- orquestração global

### `vulfram-runtime`

Responsabilidade:
- lifecycle
- filas de comandos, eventos e responses
- coordenação entre crates
- sequencing do frame/tick
- integração sistêmica final
- `RuntimeState` como orquestrador fino do sistema

Regras de state:
- cada crate define seu próprio `State` quando tiver ownership real de memória e lifecycle
- `RuntimeState` não reimplementa campos internos dos outros domínios
- recursos vivem no domínio que os consome e controla
- exemplos atuais:
- geometrias, materiais, entidades 3D e ambientes pertencem a `vulfram-realm-3d`
- texturas globais e target texture binds pertencem a `vulfram-render`
- `RuntimeState` fica com filas, deferreds, frame lifecycle e coordenação

Não deve conter:
- payloads do protocolo definidos internamente
- bindings de host
- lógica de app/demo
- semântica de cena redefinida localmente

### `vulfram-platform`

Responsabilidade:
- integração com ambiente real
- janela, canvas e event loop
- coleta de eventos brutos
- adaptação desktop/browser
- conversão de eventos brutos em input normalizado
- DPR, resize, fullscreen, pointer lock, cursor

Não deve conter:
- semântica de cena
- lógica de render
- protocolo
- estado global de runtime além do estritamente necessário para integração

### `vulfram-realm-3d`

Responsabilidade:
- câmera 3D
- luzes
- modelos
- relações espaciais 3D
- semântica 3D de recursos e componentes

Não deve conter:
- pipelines GPU
- shaders
- alocação direta de buffers GPU

### `vulfram-realm-2d`

Responsabilidade:
- semântica 2D quando essa camada existir de forma real
- câmera 2D
- sprites, layers e shapes 2D

Não deve conter:
- implementação gráfica concreta
- placeholders artificiais sem contrato mínimo

### `vulfram-bindings-*`

Responsabilidade:
- ABI/API específica de cada host
- adaptação de strings, buffers e memória
- tradução fina entre host e `vulfram-runtime`

Não devem conter:
- lógica de domínio
- orquestração complexa
- acesso direto aos subsistemas internos

### `vulfram-demo`

Responsabilidade:
- harness local
- cenas de teste
- sandbox de integração manual
- validação de features em nível de app

Não deve conter:
- lógica compartilhada de engine
- contratos públicos de bindings

---

## Regras de dependência

### Grafo de dependência desejado

```text
vulfram-types
├── vulfram-protocol
├── vulfram-input
├── vulfram-realm-core
├── vulfram-audio
└── crates semânticos auxiliares

vulfram-realm-ui   ──► vulfram-types
vulfram-realm-ui   ──► vulfram-input
vulfram-realm-ui   ──► vulfram-realm-core

vulfram-realm-3d   ──► vulfram-types
vulfram-realm-3d   ──► vulfram-realm-core

vulfram-realm-2d   ──► vulfram-types
vulfram-realm-2d   ──► vulfram-realm-core

vulfram-render     ──► vulfram-types
vulfram-render     ──► vulfram-realm-core
vulfram-render     ──► vulfram-realm-3d
vulfram-render     ──► vulfram-realm-2d
vulfram-render     ──► vulfram-realm-ui

vulfram-platform   ──► vulfram-types
vulfram-platform   ──► vulfram-input

vulfram-runtime    ──► vulfram-protocol
vulfram-runtime    ──► vulfram-types
vulfram-runtime    ──► vulfram-realm-core
vulfram-runtime    ──► vulfram-input
vulfram-runtime    ──► vulfram-realm-ui
vulfram-runtime    ──► vulfram-render
vulfram-runtime    ──► vulfram-audio
vulfram-runtime    ──► vulfram-platform
vulfram-runtime    ──► vulfram-realm-3d
vulfram-runtime    ──► vulfram-realm-2d

vulfram-bindings-* ──► vulfram-runtime
vulfram-demo       ──► vulfram-runtime
```

### Regras duras

1. `vulfram-types` não depende de ninguém
2. `vulfram-protocol` depende apenas de `vulfram-types`
3. `vulfram-input` depende apenas de `vulfram-types`
4. `vulfram-realm-core` depende apenas de `vulfram-types`
5. `vulfram-audio` depende apenas de `vulfram-types` ou abstrações neutras estritamente necessárias
6. `vulfram-realm-ui` depende de `vulfram-types`, `vulfram-input` e `vulfram-realm-core`
7. `vulfram-realm-3d` e `vulfram-realm-2d` dependem de `vulfram-types` e `vulfram-realm-core`
8. `vulfram-render` pode depender de crates semânticos, mas nenhum crate semântico pode depender de `vulfram-render`
9. `vulfram-platform` pode depender de `vulfram-input`, nunca o contrário
10. `vulfram-runtime` é o orquestrador e pode depender dos demais
11. `vulfram-bindings-*` dependem apenas de `vulfram-runtime`
12. `vulfram-demo` depende apenas do que for necessário para exercer a engine como consumidor, preferencialmente `vulfram-runtime`

### Proibições importantes

- `scene-*` não pode depender de `render`
- `input` não pode depender de `platform`
- `protocol` não pode depender de `runtime`
- `render` não pode depender de `runtime`
- `bindings-*` não podem depender diretamente de `render`, `scene-*`, `audio`, `platform` ou `ui`
- `runtime` não deve redefinir tipos já pertencentes a `types`, `protocol` ou crates semânticos

---

## Estratégia geral de implementação

A engine nova será construída por camadas, não por cópia cega do monólito atual.

### Regra central da migração

Cada crate novo deve nascer com:
- responsabilidade explícita
- API pública mínima
- testes automatizados
- documentação mínima da fronteira
- ausência de dependências indevidas

### Fonte de verdade durante a transição

- o código atual continua sendo referência funcional
- a nova arquitetura nasce em crates novos
- partes do código atual podem ser consultadas, adaptadas ou reaproveitadas
- não vamos mover o projeto inteiro para uma pasta `.old`
- a remoção do código antigo só acontece quando a parte nova equivalente estiver funcional e validada

### Critério de conclusão por fase

Uma fase só termina quando houver:
- compilação do crate novo
- testes do crate novo cobrindo o comportamento introduzido
- integração mínima com os crates já existentes
- documentação atualizada da fronteira criada ou alterada

---

## Ordem de implementação

A ordem abaixo é a ordem recomendada para execução, não apenas uma sugestão estética.

### Fase 0 — Preparação do workspace

Objetivo:
- ativar o workspace Cargo e preparar o repositório para a reconstrução por crates

Tarefas:
- criar `Cargo.toml` de workspace na raiz
- registrar `members`
- preparar diretório `crates/`
- manter o crate atual funcional durante a transição
- organizar estratégia de build e verificação do workspace

Testes e validação:
- build do estado atual ainda funcional
- `scripts/check.sh` ao final de alterações em Rust/shader relacionadas

Resultado esperado:
- workspace ativo
- sem quebra estrutural desnecessária

### Fase 1 — Criação de `vulfram-types`

Objetivo:
- estabelecer a base semântica neutra que sustentará os demais crates

Tarefas:
- criar `crates/vulfram-types`
- definir IDs lógicos e newtypes fundamentais
- mover enums e structs compartilhadas sem backend
- remover qualquer dependência em APIs concretas
- documentar o que pode e o que não pode entrar nesse crate

Testes obrigatórios:
- serialização quando aplicável
- invariantes simples de tipos
- igualdade, ordenação e hashing quando fizer parte do contrato

Critério de aceite:
- `vulfram-types` compila isoladamente
- nenhum uso de `wgpu`, `winit`, `web-sys`, bindings ou runtime logic

### Fase 2 — Criação de `vulfram-protocol`

Objetivo:
- formalizar o contrato host-runtime em um crate próprio e neutro

Tarefas:
- criar `crates/vulfram-protocol`
- definir envelopes, comandos, eventos e responses
- mover codecs MessagePack
- separar payloads serializáveis de detalhes internos do monólito atual
- evitar reaproveitar tipos internos que contaminem o contrato

Testes obrigatórios:
- round-trip serde
- round-trip MessagePack
- compatibilidade estrutural dos envelopes
- casos inválidos relevantes

Critério de aceite:
- `protocol` depende só de `types`
- nenhum payload do protocolo depende de `wgpu`, `winit`, `web-sys` ou módulos internos do runtime

### Fase 3 — Criação de `vulfram-realm-core`

Objetivo:
- consolidar a semântica de composição de cena antes da reconstrução de render e runtime

Tarefas:
- criar `crates/vulfram-realm-core`
- definir realms, surfaces, targets, target layers, connectors e presents
- definir regras determinísticas de resolução composicional
- extrair caches e índices semânticos realmente pertencentes à composição
- evitar arrastar render graphs executáveis ou tipos gráficos concretos

Testes obrigatórios:
- criação e atualização de grafos semânticos
- resolução determinística
- casos de conflito
- auto-link e índices semânticos quando aplicável

Critério de aceite:
- sem dependência em `render`
- sem `wgpu`
- fronteira de composição entendível isoladamente

### Fase 4 — Criação de `vulfram-input`

Status:
- concluída
- `vulfram-input` já concentra eventos normalizados, listeners, estado/cache de input e a semântica pura de roteamento
- o que permaneceu no core é adaptador de runtime e integração com raycast/cena/plataforma

Objetivo:
- isolar o modelo normalizado de input antes de integrar plataforma e UI

Tarefas:
- criar `crates/vulfram-input`
- modelar eventos normalizados
- definir estado e transições de keyboard, mouse, touch, pointer e gamepad
- definir foco e captura em nível genérico

Testes obrigatórios:
- normalização e transições de estado
- foco/captura
- casos de coerência temporal relevantes

Critério de aceite:
- sem DOM e sem `winit`
- tipos úteis tanto para desktop quanto browser

### Fase 5 — Criação de `vulfram-realm-ui`

Status:
- concluída no escopo semântico

Objetivo:
- reconstruir UI como subsistema semântico, separado de render e runtime

Tarefas:
- criar `crates/vulfram-realm-ui`
- definir documentos, árvore, layout e estado de interação
- integrar com `vulfram-input`
- integrar com `vulfram-realm-core` apenas onde a composição exigir

Testes obrigatórios:
- árvore e operações estruturais
- foco e navegação
- layout e atualização de estado
- bridge de input para UI

Critério de aceite:
- sem backend gráfico concreto
- sem conhecimento do runtime global

Resultado alcançado:
- `vulfram-realm-ui` já concentra contratos, documentos, árvore, estado semântico leve, foco/captura por janela, dispatch de ponteiro traçado e planejamento puro de bombeamento de eventos
- o que permanece em `src/core/ui` agora é backend concreto e integração:
  `egui`, tesselação, renderer, decode de imagens, caches gráficos e cola com runtime/janela

### Fase 6 — Criação de `vulfram-render`

Objetivo:
- reconstruir o backend gráfico concreto consumindo estado semântico externo

Tarefas:
- criar `crates/vulfram-render`
- definir recursos GPU e estado renderizável
- definir sync entre estado semântico e dados gráficos
- portar passes, pipelines, shaders e execução de frame
- manter a semântica de cena fora desse crate

Testes obrigatórios:
- testes unitários de planejamento, cache e regras puras
- testes de integração onde o custo compensar
- validações de contratos internos de render graph quando aplicável

Critério de aceite:
- `render` consome semântica, não a define
- nenhuma dependência em `runtime`

### Fase 7 — Criação de `vulfram-audio`

Objetivo:
- separar áudio como subsistema próprio, com fronteira equivalente à de render

Tarefas:
- criar `crates/vulfram-audio`
- modelar listener, sources, buffers e estado
- portar backends web/desktop
- garantir política correta de lifecycle e descarte

Status atual:
- crate criado
- `AudioState`, `AudioListenerBinding`, `AudioSourceParams`, `AudioSpatialParams`, `AudioPlayMode`, `AudioReadyEvent` e `AudioStreamState` já vivem no crate novo
- o core já consome esses tipos a partir de `vulfram-audio`
- backends e fluxo de comandos ainda permanecem no core nesta fase intermediária

Testes obrigatórios:
- regras de estado
- transporte e atualização
- casos de descarte, cancelamento e consistência

Critério de aceite:
- sem dependência em runtime global
- sem mistura com input ou render

### Fase 8 — Criação de `vulfram-runtime`

Objetivo:
- criar o orquestrador real da nova engine quando os contratos principais já existirem

Tarefas:
- criar `crates/vulfram-runtime`
- integrar lifecycle
- integrar filas de comandos, eventos e responses
- coordenar `protocol`, `realm-core`, `input`, `ui`, `render` e `audio`
- manter `runtime` como casca de coordenação e sequencing

Testes obrigatórios:
- fluxo básico de command queue
- integração entre subsistemas
- emissão de eventos e responses
- erros diagnosticáveis emitindo `SystemEvent::Error` quando aplicável

Critério de aceite:
- runtime coordena, não redefine domínios
- estado central reduzido ao que for necessário para orquestração

### Fase 9 — Criação de `vulfram-platform`

Objetivo:
- integrar ambiente real ao stack novo sem contaminar crates semânticos

Tarefas:
- criar `crates/vulfram-platform`
- portar integrações desktop/browser
- converter eventos brutos em `vulfram-input`
- lidar com window/canvas/event loop, resize, DPR, fullscreen e cursor
- etapa atual:
  - crate criado
  - planner puro de redraw por janela extraído
  - desktop já consome `vulfram-platform` para decidir redraw
  - helpers puros de browser para sizing de canvas, texto de teclado e ponteiro extraídos
  - integração browser ainda pendente

Testes obrigatórios:
- testes puros para adaptadores e conversões
- cobertura pontual de regras de integração onde possível

Critério de aceite:
- `platform` depende de `input`
- `input` não depende de `platform`

### Fase 10 — Criação de `vulfram-realm-3d`

Objetivo:
- separar explicitamente a semântica 3D quando a base composicional já estiver firme

Tarefas:
- criar `crates/vulfram-realm-3d`
- definir câmera 3D, luzes, modelos e componentes 3D
- integrar com `vulfram-realm-core`
- deixar `render` apenas consumir essas estruturas

Testes obrigatórios:
- consistência dos componentes 3D
- vínculos e índices semânticos relevantes

Critério de aceite:
- sem `wgpu`
- sem ownership gráfico

### Fase 11 — Criação de `vulfram-realm-2d`

Objetivo:
- abrir a fronteira 2D apenas quando houver contrato real suficiente

Tarefas:
- criar `crates/vulfram-realm-2d`
- definir os tipos e estruturas 2D realmente necessários
- integrar com `vulfram-realm-core`

Testes obrigatórios:
- invariantes das estruturas introduzidas
- comportamento semântico mínimo do domínio 2D

Critério de aceite:
- sem placeholders vazios sem função concreta

### Fase 12 — Criação dos `vulfram-bindings-*`

Objetivo:
- separar exports por host a partir de um `runtime` já estabilizado

Tarefas:
- criar crates dedicados para C, WASM, N-API, Python e Lua
- portar ABI/API específicas
- manter adaptação fina de buffers, ponteiros, strings e chamadas

Testes obrigatórios:
- smoke tests por binding quando viável
- testes estruturais das conversões mais críticas

Critério de aceite:
- bindings finos
- nenhuma lógica de domínio dentro deles

### Fase 13 — Criação de `vulfram-demo`

Objetivo:
- reconstruir o harness local sobre a arquitetura nova

Tarefas:
- criar `crates/vulfram-demo`
- portar cenários de demonstração
- manter Escape fechando demos por padrão
- validar integração ponta a ponta

Testes e validação:
- smoke tests do que couber automatizar
- validação manual de fluxos principais

Critério de aceite:
- demo atua como consumidor da engine, não como parte dela

### Fase 14 — Limpeza final

Objetivo:
- remover restos do desenho antigo e consolidar a arquitetura nova

Tarefas:
- apagar caminhos legados que já tenham equivalentes novos
- revisar dependências cruzadas
- revisar features
- revisar documentação do workspace
- consolidar CI e validações

Critério de aceite:
- workspace coerente
- sem duplicação estrutural desnecessária
- fronteiras de crate respeitadas

---

## Estratégia de testes por fase

### Regras gerais

- todo crate novo deve nascer com testes
- testes unitários cobrem regras puras e invariantes
- testes de integração cobrem contratos entre crates quando isso agregar segurança real
- não adiar testes para "depois da migração"
- bugs encontrados durante a migração devem gerar teste de regressão no crate correto

### Alvos por tipo de crate

- `types`: invariantes, serde e utilidades base
- `protocol`: round-trip de codec e compatibilidade estrutural
- `realm-core`: regras de composição e resolução
- `input`: transições e normalização
- `ui`: árvore, foco, layout e interação
- `render`: planejamento, caches e contratos puros; integração seletiva
- `audio`: estado e transporte
- `runtime`: integração sistêmica, filas e emissão de eventos/responses
- `bindings-*`: smoke tests e testes de adaptação crítica quando possível

---

## Regras de documentação

Ao final de cada fase concluída:

- atualizar documentação arquitetural relacionada
- registrar a fronteira e responsabilidade do crate criado ou alterado
- documentar restrições de dependência se elas não estiverem óbvias
- atualizar fluxos de integração quando houver mudança no desenho do sistema

Documentos candidatos a atualização frequente:
- `README.md`
- `docs/ARCH.md`
- `docs/REALM-ARCH.md`
- `docs/UI.md`
- documentação específica de protocolo, runtime e bindings quando criada

---

## Regras operacionais durante a implementação

- usar o código atual como referência de comportamento, não como molde estrutural obrigatório
- evitar reaproveitamento literal quando ele trouxer acoplamento antigo para o crate novo
- preferir APIs pequenas e explícitas entre crates
- remover código e tipos não usados conforme a migração avança
- quando mexer em código Rust ou shader, executar `scripts/check.sh` ao final da implementação
- tratar erros diagnosticáveis emitindo também `SystemEvent::Error` no fluxo adequado
- manter nomes e contratos coerentes com as convenções já definidas no projeto

---

## Resultado final esperado

Ao final da refatoração, o workspace deve oferecer:

- crates com fronteiras reais e auditáveis
- semântica separada de backend concreto
- runtime orquestrador, não monólito de estado
- protocolo neutro e reutilizável
- render e áudio consumindo estado externo em vez de defini-lo
- demo e bindings isolados da lógica central
- base preparada para evolução de 2D, 3D, UI, múltiplos hosts e futuras features sem crescimento desordenado

Esse plano prioriza limpeza arquitetural, incrementalismo verificável e isolamento entre crates. Se em algum ponto surgir conflito entre velocidade e fronteira saudável, a prioridade deve ser preservar a fronteira correta.
