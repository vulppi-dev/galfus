# Validation Strategy

This document defines how quality is validated in Vulfram.

## Automated Validation (CI)

Automated validation is executed in CI before matrix builds:

1. `scripts/check.sh`
2. `cargo test --lib`

`scripts/check.sh` currently runs:

1. `cargo check --lib`
2. WGSL validation (`cargo run --bin wgsl_check`)
3. `cargo fmt`

## Unit Test Coverage Scope

Unit tests focus on deterministic and pure/near-pure logic, including:

- Graph planning and ordering (`realm`, `target`)
- Graph hash/diff behavior
- Async decode manager state transitions (pending/cancel/drain)
- Report projection/aggregation helpers
- Existing UI/input/target logic tests already in the codebase

## Manual Validation Scope (Demos Required)

The following areas are not fully covered by unit tests and must be validated manually by running demos:

- Native/browser window lifecycle and interaction details
- Audio audibility/perceptual validation
- Visual quality/perceptual rendering validation (lighting, post-process, composition quality)
- Platform-specific behavior differences (desktop vs browser)

## Manual Validation Procedure

Run representative demos and validate:

1. Window open/resize/focus/close behavior
2. Input routing and interactivity
3. Audio playback/listener behavior
4. Visual pipeline output (forward/shadow/post/UI composition)
5. Runtime stability after repeated create/dispose cycles
