# Phase 0 Macro Acceptance Criteria

## Build/Test Gates
- Core Rust gates (fast reliable):
  - `cargo test -p vulfram-realm-core`
  - `cargo test -p vulfram-render`
  - `cargo test -p vulfram-runtime`
- TS gate:
  - `bunx tsc -p packages/engine/tsconfig.json`

## Migration Safety Gates
- No legacy `TargetKind` variants in public target types (`window|texture` only).
- Pointer relay disabled in runtime tick path; global pointer event stream preserved.
- Checklist-driven phase closure with one commit per closed phase.

## Demo/Behavior Smoke Gates
- Runtime ticks, UI input processing, and render loop remain functional.
- Target measurement and target upsert tests remain green.

## Exit Criteria for Phase 0
- Baseline documented.
- Inventory documented.
- Acceptance gates documented and agreed.
