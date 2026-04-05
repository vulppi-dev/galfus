# Validation Guide

This document defines the current quality-validation split between automated checks and manual verification.

## Automated Validation

Primary automated gate:

- `bun run check`
- CI workflow `build-bindings.yml`

Current scope:

- `cargo check --lib`
- WGSL shader validation
- `cargo test -p vulfram-runtime --lib` via `bun run check`
- `cargo test --workspace --lib` in CI
- formatting drift detection in CI (`git diff --exit-code` after checks)

Notes:

- This gate validates compilation and shader consistency.
- CI additionally validates that the refactored workspace still builds/tests across
  the split crates, grouped bind release packaging, and npm transport publishing workflow.
- It does not guarantee visual quality, UX behavior, or platform-policy edge cases.
- After documentation-only architecture work, this script is not required unless
  code or shader files also changed.

## Manual Validation

Manual validation is required for flows that depend on runtime/platform behavior, especially:

- Window lifecycle and platform-specific transitions
- Pointer capture semantics (`none`, `confined`, `locked`)
- Browser canvas sizing and HiDPI behavior
- Visual quality (lighting, skybox, post-process perception)
- Audio audibility and interaction patterns

Recommended approach:

1. Run focused demos/scenarios that exercise the changed subsystem.
2. Validate behavior on at least one desktop backend and wasm/browser when applicable.
3. Confirm host-observable events (`WindowEvent`, `SystemEvent::Error`) for diagnosable failures.

## Rule of Thumb

If a change affects rendering perception, OS/browser policy behavior, or timing-sensitive interactions, it must include manual validation in addition to `bun run check`.

## Bun Workspace

The repository now includes a Bun workspace at the root for transport packages
under `packages/`.

Preferred validation entrypoint:

- `bun run check`

That script runs the standard Rust/WGSL validation pipeline previously handled
by the old shell-based check script.
