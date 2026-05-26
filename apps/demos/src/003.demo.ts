import {
  Mount,
  World2D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 6_000;
const FRAME_TARGET_MS = 16;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus Demo 003 - Realm2D',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    transparent: false,
    initialState: 'maximized'
  });

  let totalMs = 0;

  const worldId = World2D.create2DWorld();
  Mount.mountWorld(worldId, { target: { kind: 'window', windowId } });

  const quadGeometryA = World2D.create2DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo003QuadA'
  });
  const quadGeometryB = World2D.create2DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo003QuadB'
  });

  const matRed = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo003Red',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.95, 0.25, 0.35, 1.0]
      }
    }
  });

  const matBlue = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo003Blue',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.45, 0.95, 1.0]
      }
    }
  });

  const camera = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, camera, { position: [0, 0, 5] });
  World2D.create2DCamera(worldId, camera, {
    kind: 'orthographic',
    near: 0.01,
    far: 100.0,
    orthoScale: 2.5,
    order: 0
  });

  const spriteA = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, spriteA, {
    position: [-0.8, 0.0, 0.0],
    scale: [0.9, 0.9, 1.0]
  });
  World2D.create2DSprite(worldId, spriteA, {
    geometryId: quadGeometryA,
    materialId: matRed,
    layer: 1
  });

  const spriteB = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, spriteB, {
    position: [0.9, -0.5, 0.0],
    scale: [0.7, 0.7, 1.0]
  });
  World2D.create2DSprite(worldId, spriteB, {
    geometryId: quadGeometryB,
    materialId: matBlue,
    layer: 2
  });

  const shapeA = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, shapeA, {
    position: [0.0, 0.35, 0.0],
    scale: [0.5, 1.4, 1.0]
  });
  World2D.create2DShape(worldId, shapeA, {
    geometryId: quadGeometryA,
    materialId: matRed,
    layer: 0
  });

  const start = performance.now();
  let last = start;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const t = totalMs / 1000;
    const xA = Math.sin(t * 1.2) * 0.55;
    const yB = Math.cos(t * 1.8) * 0.35;
    const rot = t * 1.7;

    const rotA = quat.fromEuler(quat.create(), 0, 0, (rot * 180) / Math.PI);
    const rotShape = quat.fromEuler(quat.create(), 0, 0, (-rot * 0.7 * 180) / Math.PI);

    World2D.update2DTransform(worldId, spriteA, {
      position: [xA, 0.0, 0.0],
      rotation: [rotA[0], rotA[1], rotA[2], rotA[3]],
      scale: [0.9, 0.9, 1.0]
    });
    World2D.update2DTransform(worldId, spriteB, {
      position: [0.9, yB, 0.0],
      scale: [0.7, 0.7, 1.0]
    });
    World2D.update2DTransform(worldId, shapeA, {
      position: [0.0, 0.35, 0.0],
      rotation: [rotShape[0], rotShape[1], rotShape[2], rotShape[3]],
      scale: [0.5, 1.4, 1.0]
    });
    World2D.update2DTransform(worldId, camera, {
      position: [Math.sin(t * 0.4) * 0.1, 0.0, 5.0]
    });

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((resolve) =>
      setTimeout(resolve, Math.max(0, FRAME_TARGET_MS - frameElapsed))
    );
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
