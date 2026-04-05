import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick,
} from '@vulfram/engine';
import {
  attachCollisionAabb,
  createPerspectiveRhZo,
  drawCollisionAabbGizmo,
  raycastPointerCollisionAabb,
} from '@vulfram/engine/helpers';
import { mat4, quat } from 'gl-matrix';
import { transportBunFfi } from '@vulfram/transport-bun';

const RUN_DURATION_MS = 60_000;
const FRAME_TARGET_MS = 16;
const KeyCode = {
  Escape: 106,
  KeyQ: 35,
  KeyE: 23,
  KeyW: 41,
  KeyS: 37,
  KeyA: 19,
  KeyD: 22,
} as const;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Vulfram Input Test - Rotate Cube (Keyboard + Pointer)',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    transparent: false,
  });
  let totalMs = 0;

  const worldId = World3D.create3DWorld();
  Mount.mountWorld(worldId, { target: { kind: 'window', windowId } });

  World3D.configure3DEnvironment(worldId, {
    msaa: { enabled: true, sampleCount: 1 },
    skybox: {
      mode: 'none',
      intensity: 0,
      rotation: 0,
      groundColor: [0.02, 0.03, 0.04],
      horizonColor: [0.12, 0.16, 0.22],
      skyColor: [0.2, 0.35, 0.6],
      cubemapTextureId: null,
    },
    clearColor: [0.02, 0.02, 0.03, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1.05,
      filterGamma: 2,
      filterSaturation: 1,
      filterContrast: 1,
      filterVignette: 0,
      filterGrain: 0,
      filterChromaticAberration: 0,
      filterBlur: 0,
      filterSharpen: 0,
      filterTonemapMode: 1,
      outlineEnabled: false,
      outlineStrength: 0,
      outlineThreshold: 0.2,
      outlineWidth: 1,
      outlineQuality: 1,
      filterPosterizeSteps: 0,
      cellShading: false,
      ssaoEnabled: false,
      ssaoStrength: 1,
      ssaoRadius: 0.75,
      ssaoBias: 0.025,
      ssaoPower: 1.5,
      ssaoBlurRadius: 2,
      ssaoBlurDepthThreshold: 0.02,
      bloomEnabled: false,
      bloomThreshold: 1,
      bloomKnee: 0.5,
      bloomIntensity: 0.8,
      bloomScatter: 0.7,
    },
  });

  const blueTexId = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.3, 0.6, 0.9, 1] },
    srgb: true,
    label: 'Blue Texture',
  });
  const redTexId = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.95, 0.2, 0.2, 1] },
    srgb: true,
    label: 'Red Texture',
  });
  const blueMatId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Cube Material Blue',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: blueTexId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });
  const redMatId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Cube Material Red',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: redTexId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });
  const geoId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube',
  });

  const camEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, camEnt, { position: [0, 0, 5] });
  World3D.create3DCamera(worldId, camEnt, {
    kind: 'perspective',
    near: 0.1,
    far: 100.0,
    order: 0,
  });

  const lightEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEnt, { position: [5, 5, 5] });
  World3D.create3DLight(worldId, lightEnt, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 9,
    range: 24,
    castShadow: true,
  });

  const cubePivotEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubePivotEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });

  const cubeBlueEnt = World3D.create3DEntity(worldId);
  World3D.set3DParent(worldId, cubeBlueEnt, cubePivotEnt);
  World3D.update3DTransform(worldId, cubeBlueEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DModel(worldId, cubeBlueEnt, {
    geometryId: geoId,
    materialId: blueMatId,
    castShadow: true,
    receiveShadow: true,
  });

  const cubeRedEnt = World3D.create3DEntity(worldId);
  World3D.set3DParent(worldId, cubeRedEnt, cubePivotEnt);
  World3D.update3DTransform(worldId, cubeRedEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [0.0001, 0.0001, 0.0001],
  });
  World3D.create3DModel(worldId, cubeRedEnt, {
    geometryId: geoId,
    materialId: redMatId,
    castShadow: true,
    receiveShadow: true,
  });

  const collider = attachCollisionAabb({
    worldId,
    modelEntityId: cubePivotEnt,
    halfExtents: [0.5, 0.5, 0.5],
    debugGizmoAabb: true,
    name: 'demo003-cube-collider',
  });

  const start = performance.now();
  let elapsedMs = 0;
  const rotation: [number, number, number] = [0, 0, 0];
  const pointerDragSpeed = 1.8;
  let isColliding = false;
  // Prime input/state mirrors before entering the frame loop.
  tick(totalMs, FRAME_TARGET_MS);
  while (elapsedMs < RUN_DURATION_MS) {
    const frameStart = performance.now();

    if (
      World3D.is3DWindowCloseRequested(worldId) ||
      World3D.is3DKeyPressed(worldId, KeyCode.Escape)
    ) {
      break;
    }

    const rotDelta = 2.0 * (FRAME_TARGET_MS / 1000);
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyQ)) rotation[1] -= rotDelta;
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyE)) rotation[1] += rotDelta;
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyW)) rotation[0] -= rotDelta;
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyS)) rotation[0] += rotDelta;
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyA)) rotation[2] -= rotDelta;
    if (World3D.is3DKeyPressed(worldId, KeyCode.KeyD)) rotation[2] += rotDelta;

    if (World3D.is3DPointerButtonPressed(worldId, 0)) {
      const targetDelta = World3D.get3DPointerTargetDelta(worldId);
      const delta = targetDelta ?? World3D.get3DPointerDelta(worldId);
      rotation[1] += delta[0] * pointerDragSpeed * (FRAME_TARGET_MS / 1000);
      rotation[0] += delta[1] * pointerDragSpeed * (FRAME_TARGET_MS / 1000);
    }

    // Apply pivot transform before collider/gizmo queries so everything in this
    // frame uses the same resolved world transform.
    const q = quat.create();
    quat.fromEuler(
      q,
      (rotation[0] * 180) / Math.PI,
      (rotation[1] * 180) / Math.PI,
      (rotation[2] * 180) / Math.PI,
    );
    World3D.update3DTransform(worldId, cubePivotEnt, {
      rotation: [q[0], q[1], q[2], q[3]],
    });

    const pointerWindow = World3D.get3DPointerPosition(worldId);
    const pointerWindowSize = World3D.get3DPointerWindowSize(worldId);
    const [fallbackWindowWidth, fallbackWindowHeight] = World3D.get3DWindowSize(worldId);
    let viewportLog = 'none';
    let frameCollision = false;
    if (pointerWindow) {
      const viewportSize =
        pointerWindowSize ?? [fallbackWindowWidth, fallbackWindowHeight];
      const [viewportWidth, viewportHeight] = viewportSize;
      viewportLog = `${viewportWidth}x${viewportHeight}`;
      const view = mat4.lookAt(mat4.create(), [0, 0, 5], [0, 0, 0], [0, 1, 0]);
      // Exact match with core camera projection (glam::perspective_rh) in reverse-Z.
      const projection = createPerspectiveRhZo(
        (45 * Math.PI) / 180,
        viewportWidth / Math.max(1, viewportHeight),
        100,
        0.1,
      );
      const hit = raycastPointerCollisionAabb({
        collision: collider,
        pointerEvent: {
          position: pointerWindow,
          windowWidth: viewportWidth,
          windowHeight: viewportHeight,
        },
        viewMatrix: view,
        projectionMatrix: projection,
        fallbackViewportSize: [fallbackWindowWidth, fallbackWindowHeight],
        edgePaddingPixels: 0.75,
      });
      frameCollision = hit !== null;
    }

    if (frameCollision !== isColliding) {
      isColliding = frameCollision;
      if (isColliding) {
        World3D.update3DTransform(worldId, cubeBlueEnt, {
          scale: [0.0001, 0.0001, 0.0001],
        });
        World3D.update3DTransform(worldId, cubeRedEnt, {
          scale: [1, 1, 1],
        });
      } else {
        World3D.update3DTransform(worldId, cubeBlueEnt, {
          scale: [1, 1, 1],
        });
        World3D.update3DTransform(worldId, cubeRedEnt, {
          scale: [0.0001, 0.0001, 0.0001],
        });
      }
    }
    drawCollisionAabbGizmo(collider);

    if (elapsedMs % 2000 < FRAME_TARGET_MS) {
      console.log(
        `Demo003: pointerWindow=${pointerWindow ? `${pointerWindow[0].toFixed(1)},${pointerWindow[1].toFixed(1)}` : 'none'} viewport=${viewportLog} colliding=${isColliding}`,
      );
    }

    // Flush this frame's transform + gizmo intents to core in the same loop pass.
    tick(totalMs + elapsedMs, FRAME_TARGET_MS);

    await new Promise((r) => setTimeout(r, FRAME_TARGET_MS));
    elapsedMs = performance.now() - start;
  }

  closeWindow(windowId);
  for (let i = 0; i < 10; i++) {
    tick(totalMs + elapsedMs + i * FRAME_TARGET_MS, FRAME_TARGET_MS);
    await new Promise((r) => setTimeout(r, FRAME_TARGET_MS));
  }
  disposeEngine();
}

main().catch((err) => {
  console.error('Test failed:', err);
  process.exit(1);
});
