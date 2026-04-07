# Vulfram Engine Overview

Vulfram Engine is the TypeScript host runtime for the Vulfram core. It manages worlds, entities, and resources, then sends batched commands to the native/WASM core via a transport.

## Core Concepts

- **Engine**: Global runtime that owns transports, worlds, and the execution pipeline.
- **World**: A logical scene bound to a window ID. Most commands include a `windowId`.
- **Entities**: Lightweight IDs in the host ECS; components are plain data.
- **Resources**: GPU-backed objects created in the core (materials, textures, geometry).
- **Intents**: Host-side requests queued and processed on `tick()`.
- **Systems**: Functions that translate intents and synchronize with the core.

## Data Flow

1. Host enqueues intents by calling API helpers (create entity, update transform, etc.).
2. On `tick()`, systems translate intents into core commands.
3. Commands are batched and sent to the core.
4. The core replies with responses and emits events (input/window/system).

## Serialization Contract

- The host sends sparse intent whenever the core already owns the default.
- TypeScript command types aim to mirror the core serialization shape closely, especially for nested UI/audio payloads, render graph ids, bytes, and fixed-size matrices.
- Repeated mergeable commands in the same frame are compacted before serialization so the core receives only the latest effective patch.
- Internal vector and matrix fallback values are initialized directly through `gl-matrix` to keep math-shaped defaults consistent across systems.

## Where to Start

- `docs/getting-started.md` for a minimal setup.
- `docs/resources.md` for materials, textures, geometry, and models.
- `docs/render-graphs.md` for custom render graph catalog commands.
- `docs/events-input.md` for input handling and window state queries.
- `docs/input-frame-contract.md` for frame-order/reset semantics of mirrored input.
- `docs/input-inventory-phase-0.md` for baseline ownership/read-write mapping.
- `docs/input-core-extraction-phase-1.md` for the first extraction step of input core modules.
- `docs/input-routing-extraction-phase-2.md` for routed pointer extraction by world/target scope.
- `docs/world-input-adapter-phase-3.md` for world facade adapters and `World3D` input split.
- `docs/world3d-api-completion-phase-4.md` for completed `World3D` input/diagnostics query wrappers.
- `docs/consumer-alignment-phase-5.md` for dependent package/demo alignment after API completion.
- `docs/legacy-cleanup-phase-6.md` for convergence cleanup and removal of redundant legacy paths.
- `docs/input-architecture-final.md` for the consolidated final input architecture.
- `docs/input-usage-guide.md` for practical `World3D` input usage examples.
- `docs/input-hardening-phase-7.md` for final hardening checklist/status.
- `docs/rendering-environment.md` for skybox, MSAA, and shadows.
