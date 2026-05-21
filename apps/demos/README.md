# Galfus TypeScript Demos

This package now keeps a single canonical demo: `001` (FrameGraph baseline parity).

## Local build prerequisites

Build local core bindings before running demos:

```bash
bun run build:local
```

Main script: `scripts/build-local-bindings.ts`

Supported flags:

- `--mode debug|release`
- `--skip-bun`
- `--skip-napi`
- `--skip-wasm`

Expected outputs:

- `packages/transport-bun/dist/<platform>/...`
- `packages/transport-napi/dist/<platform>/...`
- `packages/transport-browser/dist/galfus_core*.wasm/js`

Operational prerequisites:

- Rust toolchain installed
- `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` (auto-installed by script when missing)

Validation script for repo checks:

```bash
bun run check
```

## Run Demo 001 (CLI Bun)

```bash
bun run --cwd apps/demos demo 001
```

## Run Demo 001 (Web/WASM)

```bash
bun run demo:web
```

Open with hash `#demo-001` (default is already `001`).
