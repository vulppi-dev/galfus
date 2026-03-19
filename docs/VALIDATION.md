# Validation Guide

This document defines the current quality-validation split between automated checks and manual verification.

## Automated Validation

Primary automated gate:

- `scripts/check.sh`

Current scope:

- `cargo check --lib`
- WGSL shader validation

Notes:

- This gate validates compilation and shader consistency.
- It does not guarantee visual quality, UX behavior, or platform-policy edge cases.

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

If a change affects rendering perception, OS/browser policy behavior, or timing-sensitive interactions, it must include manual validation in addition to `scripts/check.sh`.
