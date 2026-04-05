# Vulfram Documentation Index

## Start Here

- high-level overview: [OVERVIEW.md](OVERVIEW.md)
- public ABI contract: [ABI.md](ABI.md)
- current runtime architecture: [ARCH.md](ARCH.md)
- internal Rust architecture: [API.md](API.md)
- realm composition architecture: [REALM-ARCH.md](REALM-ARCH.md)
- render graph resource format: [RENDER-GRAPH.md](RENDER-GRAPH.md)
- terminology: [GLOSSARY.md](GLOSSARY.md)
- validation policy: [VALIDATION.md](VALIDATION.md)

## For Binding Authors and Integrators

Read in this order:

1. [OVERVIEW.md](OVERVIEW.md)
2. [ABI.md](ABI.md)
3. [ARCH.md](ARCH.md)
4. [cmds/](cmds/)
5. [GLOSSARY.md](GLOSSARY.md)

Important current truths:

- the host composes through `Realm`, `Target` and `TargetLayer`
- `Surface`, `Present` and `Connector` are internal runtime tables
- render graphs are global resources bound per realm

## JavaScript Workspace

- transport packages live under `packages/transport-*`
- the source-first engine packages live under `packages/engine`,
  `packages/gltf-loader` and `packages/camera-control`
- browser-facing demos live under `apps/demos`

## For Core Contributors

Read in this order:

1. [OVERVIEW.md](OVERVIEW.md)
2. [ARCH.md](ARCH.md)
3. [API.md](API.md)
4. [REALM-ARCH.md](REALM-ARCH.md)
5. [RENDER-GRAPH.md](RENDER-GRAPH.md)
6. [GLOSSARY.md](GLOSSARY.md)

## Notes

- older planning assumptions that treated `SurfaceId`, `PresentId` and
  `ConnectorId` as host-managed should be considered obsolete
- `vulfram-runtime` is the current integration root of the Rust side
- `vulfram-render` is the home of rendering policy and the preferred long-term
  home of auto-graph planning policy
