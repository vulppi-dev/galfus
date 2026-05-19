# Realm Architecture

This document defines the active composition architecture used by Vulfram vNext.

## Core primitives

- `Realm`
- `Target`
- `Layer`
- `Texture`
- `FrameGraph`
- `RenderInvocation`

Removed from public architecture:

- `WidgetRealmViewport`
- `RealmPlane`
- `Connector`
- `Present`
- `Surface` as public composition concept

## Realm

A `Realm` is a logical render domain.

- `realm-3d`: scene, cameras, lights, materials, models, internal `Graph3D`
- `realm-2d`: scene, sprites/shapes, internal `Graph2D`

A realm is not a window and not a texture. It has no fixed output size and no clear policy.

The same realm can be rendered multiple times in the same frame through different target layers.

## Target

A `Target` is a final composition destination.

Kinds:

- `window`
- `texture`

A target owns:

- `kind`
- resolved size
- clear policy
- list of layers

Clear happens at target scope only.

## Layer

A `Layer` is a composition instruction inside one target.

It defines:

- which realm to render
- where (`rect`)
- how to compose (`blend`, `opacity`)
- whether it is active (`enabled`)
- visual ordering (`zIndex`, with deterministic tie-break by insertion key)

A layer does not embed a realm and does not own clear state.

## RenderInvocation

Each concrete layer execution becomes one `RenderInvocation`.

`RenderInvocation` carries runtime-only execution context:

- `realm`
- `target`
- `layer`
- `resolved_rect_px`
- `render_size_px`
- `frame_id`

This guarantees that one realm rendered in multiple places/sizes does not share ambiguous per-frame size state.

## FrameGraph

`FrameGraph` is the global per-frame scheduler.

It resolves:

- active targets
- resolved layers
- produced textures
- consumed textures
- dependency edges between targets
- deterministic target execution order

Cycles are handled by using previous-frame cached texture data on cyclic reads.

## Composition flow

1. Resolve active targets.
2. Resolve layer rects against target size.
3. Build render invocations.
4. Build target dependency graph from texture production/consumption.
5. Order targets deterministically.
6. For each target: clear, render each layer invocation, compose layer image.

## Graph separation

There are two graph levels:

- `FrameGraph`: orders targets/layers/textures globally
- `Graph3D` / `Graph2D`: orders passes inside one render invocation

These levels are independent and complementary.
