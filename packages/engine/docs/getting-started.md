# Getting Started

This is a minimal example using the current public API (`core`, `window`, `mount`, `world3d`).

```ts
import { initEngine, tick } from '@vulfram/engine/core';
import { createWindow } from '@vulfram/engine/window';
import { mountWorldToWindow } from '@vulfram/engine/mount';
import * as World3D from '@vulfram/engine/world3d';
import { transportWasm } from '@vulfram/transport-browser';

initEngine({ transport: transportWasm });

const worldId = World3D.create3DWorld();
const { windowId } = createWindow({
  title: 'Hello Vulfram',
  size: [1024, 640]
});

mountWorldToWindow(worldId, windowId);

const camera = World3D.create3DEntity(worldId);
World3D.update3DTransform(worldId, camera, {
  position: [0, 1.2, 4]
});
World3D.create3DCamera(worldId, camera, {
  kind: 'perspective',
  near: 0.1,
  far: 100,
  order: 0
});

const light = World3D.create3DEntity(worldId);
World3D.create3DLight(worldId, light, {
  kind: 'point',
  color: [1, 1, 1],
  intensity: 8,
  range: 20
});
World3D.update3DTransform(worldId, light, {
  position: [2, 4, 2]
});

const geometryId = World3D.create3DGeometry(worldId, {
  type: 'primitive',
  shape: 'cube'
});
const materialId = World3D.create3DMaterial(worldId, {
  kind: 'standard',
  options: {
    type: 'standard',
    content: {
      baseColor: [1, 1, 1, 1],
      surfaceType: 'opaque',
      flags: 0
    }
  }
});

const cube = World3D.create3DEntity(worldId);
World3D.create3DModel(worldId, cube, {
  geometryId,
  materialId
});

let last = performance.now();
function frame(now: number) {
  const delta = now - last;
  last = now;
  tick(now, delta);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
```

## Notes

- Call `initEngine()` once before any other API calls.
- World mounting is async-friendly internally; for strict boot sequencing you can poll with `waitWorldReady(...)` from `@vulfram/engine/mount`.
- `tick()` must be called once per frame with a monotonic timestamp.
- Most command-style helpers now support sparse payloads; omitted fields are not serialized and fall back to core defaults when supported.
- Environment and shadow configuration helpers also strip values that already match core defaults, so repeated configuration tends to serialize as true deltas only.
- Primitive geometry and material create helpers also omit default-equivalent option payloads when the core can resolve them on its own.
- The exported TS command types are now stricter around nested UI/audio payloads, render graph ids, render-graph param maps, byte arrays, and fixed-size matrices, matching the core contract more closely than the older permissive shapes.
- Internally, engine-side math defaults are initialized through `gl-matrix` helpers so transforms and vector-shaped fallback values stay consistent without repeating array literals by hand.
- Repeated same-frame model/camera/light and target-layer upserts are compacted before batch serialization so only the latest effective payload reaches the core.
