# @galfus/engine

Functional engine for the Galfus core, focused on a simple API for creating worlds, entities, and mounting to targets.

## Installation

```bash
bun add @galfus/engine @galfus/transport-browser
```

Use the transport that fits your environment (`transport-browser`, `transport-bun`, `transport-napi`).

## Simple example

```ts
import { initEngine, tick } from '@galfus/engine/core';
import { createWindow } from '@galfus/engine/window';
import { mountWorldToWindow } from '@galfus/engine/mount';
import * as World3D from '@galfus/engine/world3d';
import { transportWasm } from '@galfus/transport-browser';

initEngine({ transport: transportWasm });

const world = World3D.create3DWorld();
const { windowId } = createWindow({
  title: 'Galfus Engine - Simple Demo',
  size: [1280, 720]
});

mountWorldToWindow(world, windowId);

const camera = World3D.create3DEntity(world);
World3D.create3DCamera(world, camera, {
  kind: 'perspective',
  near: 0.1,
  far: 100,
  order: 0
});
World3D.update3DTransform(world, camera, {
  position: [0, 1.2, 4]
});

const light = World3D.create3DEntity(world);
World3D.create3DLight(world, light, {
  kind: 'directional',
  color: [1, 1, 1],
  intensity: 2
});
World3D.update3DTransform(world, light, {
  position: [3, 6, 2]
});

const geom = World3D.create3DGeometry(world, { type: 'primitive', shape: 'cube' });
const mat = World3D.create3DMaterial(world, {
  kind: 'standard',
  options: {
    type: 'standard',
    content: {
      baseColor: [0.9, 0.2, 0.2, 1],
      surfaceType: 'opaque',
      flags: 0
    }
  }
});

const cube = World3D.create3DEntity(world);
World3D.create3DModel(world, cube, { geometryId: geom, materialId: mat });
World3D.update3DTransform(world, cube, {
  position: [0, 0, 0]
});

let last = performance.now();
function frame(now: number) {
  const dt = now - last;
  last = now;
  tick(now, dt);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
```

## Public module structure

- `@galfus/engine/core`: init, tick, dispose, system/component registration
- `@galfus/engine/window`: window APIs
- `@galfus/engine/world3d`: 3D world APIs
- `@galfus/engine/mount`: world binding to targets/windows
- `@galfus/engine/ecs`: ECS types
- `@galfus/engine/math`: bundled vector/quaternion/matrix helpers
- `@galfus/engine/types`: command/event types

## Documentation

Defaults and sparse command payloads:

- `createWindow()` now accepts sparse props; omitted fields use core defaults.
- `configure3DEnvironment()` and `configure3DShadows()` accept partial configs and only send fields that differ from core defaults.
- Core-backed create commands for camera/light/model/texture/audio accept more omitted fields, reducing serialized payload size.
- Primitive geometry creation now omits `options` entirely when the resolved shape config matches the core default, and material creation omits empty option payloads.
- Notifications no longer force host-generated ids or timeout defaults; the core owns those defaults.
- The principal `@galfus/engine` binding no longer exposes UI commands/events.
- Resource state authority remains in core (`get/list` for reads), while host-local ECS state is intentionally kept for runtime systems (interpolation, transforms, animation and scene sync) to avoid unnecessary command roundtrips.
- Internal vector and matrix defaults in the engine now come directly from the bundled `@galfus/engine/math` module, with the vendored math runtime living inside the engine package so math-shaped defaults stay consistent across systems.
- Before each batch is serialized, mergeable scene and target-layer upserts are compacted so repeated same-frame updates send only the latest effective patch.

Temporary documentation URL:

- https://vulppi.dev/galfus/docs

When the final engine documentation URL is available, this README will be updated.
