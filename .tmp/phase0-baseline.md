# Phase 0 Baseline

## Branch
- `dev/replace-arch`

## Baseline Commit
- `1e29e45`

## Baseline Timestamp
- `2026-05-15` (local workspace)

## Rollback Reference
- Rollback target for this phase: `git checkout 1e29e45`
- Scope note: this rollback reference is for Phase 0 planning artifacts only.

## Working Assumptions
- Host logical IDs remain host-owned and unique.
- vNext migration is direct breaking change (no long compat window).
- Pointer routing relay is being removed in favor of global pointer stream only.
