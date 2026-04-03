# Vulfram Glossary

## Host

External application/runtime that calls the exported `vulfram_*` functions.

The host owns:

- game/application logic
- logical IDs
- command batches
- frame driving
- response/event consumption

## Core

The Rust side of Vulfram as integrated by `vulfram-runtime` and its dependent
crates.

## Runtime

Current integration root of the Rust side.

In practice this means `vulfram-runtime` is where:

- ABI entry points are re-exported
- commands/events/responses are orchestrated
- frame lifecycle is coordinated
- subsystem states are integrated

## Render Policy

Rules that directly determine renderability, composition, sizing, pass order,
surface planning and WGPU-facing execution.

In Vulfram, render policy belongs primarily to `vulfram-render`.

## Realm

Execution scope for rendering.

A realm has:

- a `kind`
- an optional `render_graph_id`
- an internal output surface resolved by the core

## Target

Host-visible logical output anchor.

Examples:

- window
- widget realm viewport
- realm plane
- texture

## TargetLayer

Host-visible mapping from one realm to one target plus layout/composition data.

This is the host-facing composition API.

## Surface

Core-owned runtime table representing a renderable/sampleable output.

Important:

- not directly created by the host
- derived internally from realm/target composition

## Present

Core-owned mapping from a window root to a surface.

## Connector

Core-owned mapping used to compose one realm surface into another realm/window
host context.

## Auto-Graph

The reconciliation process that derives internal composition tables and graph
diagnostics from host-provided `Target` and `TargetLayer` maps.

Recommended ownership:

- policy in `vulfram-render`
- command application in `vulfram-runtime`
- DTOs/state semantics in `vulfram-realm-core`

## UniversalState

Current broad runtime aggregate in `vulfram-runtime`.

It is realm-centric but not realm-only. It currently mixes:

- composition
- targets
- input routing
- UI state
- scene/resource registries
- render graph catalogs kept in a dedicated `render_catalog` sub-state
- diagnostics

Because of that, it should be split before any attempt to move it into
`vulfram-realm-core`.

## Render Graph

Global render graph resource referenced by logical `render_graph_id` and bound
per realm.

## Logical IDs

Host-managed IDs such as:

- window IDs
- realm IDs
- target IDs
- resource IDs
- component IDs
- UI IDs

The host guarantees validity and uniqueness.

## Internal IDs / Handles

Core-owned identifiers/handles such as:

- `SurfaceId`
- `PresentId`
- `ConnectorId`
- GPU resources
- compiled plans and caches
