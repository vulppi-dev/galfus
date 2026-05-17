# Vulfram Documentation Index

## Start Here

- high-level overview: [OVERVIEW.md](OVERVIEW.md)
- public ABI contract: [ABI.md](ABI.md)
- current runtime architecture: [ARCH.md](ARCH.md)
- internal API snapshot: [API.md](API.md)
- realm architecture (vNext): [REALM-ARCH.md](REALM-ARCH.md)
- render graph model (vNext): [RENDER-GRAPH.md](RENDER-GRAPH.md)
- canonical examples (vNext): [EXAMPLES-VNEXT.md](EXAMPLES-VNEXT.md)
- terminology: [GLOSSARY.md](GLOSSARY.md)
- command reference: [cmds/](cmds/)

## For Binding Authors and Integrators

Read in this order:

1. [OVERVIEW.md](OVERVIEW.md)
2. [ABI.md](ABI.md)
3. [REALM-ARCH.md](REALM-ARCH.md)
4. [RENDER-GRAPH.md](RENDER-GRAPH.md)
5. [EXAMPLES-VNEXT.md](EXAMPLES-VNEXT.md)
6. [cmds/](cmds/)

Current baseline rules:

- host composition uses `Realm`, `Target`, `TargetLayer`
- targets are only `window` and `texture`
- pointer flow is global (no target-routed relay)
- materials are unified as `ShaderMaterial` (`preset: standard|pbr`)

## For Core Contributors

Read in this order:

1. [ARCH.md](ARCH.md)
2. [API.md](API.md)
3. [REALM-ARCH.md](REALM-ARCH.md)
4. [RENDER-GRAPH.md](RENDER-GRAPH.md)
5. [GLOSSARY.md](GLOSSARY.md)
6. [EXAMPLES-VNEXT.md](EXAMPLES-VNEXT.md)

## Release / Validation Supporting Docs

- [VALIDATION.md](VALIDATION.md)
- [CI-RELEASE-PUBLISH.md](CI-RELEASE-PUBLISH.md)
- [OIDC-PUBLISH-SETUP.md](OIDC-PUBLISH-SETUP.md)
