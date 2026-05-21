import { Mount, World3D, createWindow, initEngine, tick } from '@galfus/engine';
import type { World3DId } from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { initWasmTransport, transportWasm } from '@galfus/transport-browser';

const FRAME_MS = 16;
const SHADOW_CONFIG = {
  tileResolution: 1024,
  atlasTilesW: 16,
  atlasTilesH: 16,
  atlasLayers: 2,
  virtualGridSize: 1,
  smoothing: 2,
  normalBias: 0.05
} as const;

type DemoUpdate = (dtSeconds: number) => void;

function getDemoId(): string {
  const hash = window.location.hash.replace('#', '');
  if (hash.startsWith('demo-')) return hash.replace('demo-', '');
  return '001';
}

function setupCommonWindow(): { worldId: World3DId } {
  const { windowId } = createWindow({
    title: 'Galfus Demo 001 - FrameGraph',
    size: [1280, 720],
    position: [0, 0],
    resizable: true,
    canvasId: 'galfus-canvas'
  });

  const worldId = World3D.create3DWorld();
  Mount.mountWorld(worldId, { target: { kind: 'window', windowId } });

  return { worldId };
}

function setupDemo001(worldId: World3DId): DemoUpdate {
  World3D.configure3DEnvironment(worldId, {
    clearColor: [0, 0, 0, 1],
    post: {
      outlineEnabled: true,
      outlineStrength: 0.5,
      outlineThreshold: 0.25,
      outlineWidth: 1.25
    }
  });
  World3D.configure3DShadows(worldId, SHADOW_CONFIG);

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo001Cube'
  });
  const floorGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo001FloorPlane'
  });

  const standardMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Standard',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.92, 0.35, 0.32, 1],
        renderSide: 'back',
        surfaceType: 'opaque',
        flags: 0
      }
    }
  });

  const pbrMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'pbr',
    label: 'Demo001Pbr',
    options: {
      type: 'pbr',
      content: {
        baseColor: [0.25, 0.86, 0.62, 1],
        metallic: 0.55,
        roughness: 0.35,
        ao: 1,
        normalScale: 1,
        renderSide: 'back',
        flags: 0,
        surfaceType: 'opaque'
      }
    }
  });

  const customSimpleMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001CustomSimplePlaceholder',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.25, 0.45, 0.98, 1],
        emissiveColor: [0, 0, 0, 0],
        specColor: [1, 1, 1, 1],
        specPower: 64,
        renderSide: 'back',
        surfaceType: 'opaque',
        flags: 0
      }
    }
  });

  const floorMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Floor',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.24, 0.24, 0.26, 1],
        specColor: [0.05, 0.05, 0.05, 1],
        specPower: 8,
        renderSide: 'double-side',
        surfaceType: 'opaque',
        flags: 0
      }
    }
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2, 7],
    rotation: [-0.1305262, 0, 0, 0.9914449]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 120,
    order: 0
  });

  const lightEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEntity, {
    position: [3, 5, 5],
    rotation: [0, 0, 0, 1]
  });
  World3D.create3DLight(worldId, lightEntity, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 4,
    range: 24,
    castShadow: true
  });

  const cubeA = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubeA, {
    position: [-2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cubeA, {
    geometryId: cubeGeometryId,
    materialId: standardMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  const cubeB = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubeB, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cubeB, {
    geometryId: cubeGeometryId,
    materialId: pbrMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  const cubeC = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubeC, {
    position: [2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cubeC, {
    geometryId: cubeGeometryId,
    materialId: customSimpleMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  const floor = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, floor, {
    position: [0, -1, 0],
    rotation: [-0.7071068, 0, 0, 0.7071068],
    scale: [20, 20, 1]
  });
  World3D.create3DModel(worldId, floor, {
    geometryId: floorGeometryId,
    materialId: floorMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  let elapsedSeconds = 0;
  let nextShadowRefreshSeconds = 0;
  return (dtSeconds) => {
    elapsedSeconds += dtSeconds;

    const angleA = elapsedSeconds * 1.8;
    const angleB = elapsedSeconds * 2.5 + 0.6;
    const angleC = elapsedSeconds * 1.4 + 1.2;
    const qa = quat.fromEuler(quat.create(), 0, (angleA * 180) / Math.PI, 0);
    const qb = quat.fromEuler(quat.create(), 0, (angleB * 180) / Math.PI, 0);
    const qc = quat.fromEuler(quat.create(), 0, (angleC * 180) / Math.PI, 0);

    World3D.update3DTransform(worldId, cubeA, { rotation: [qa[0], qa[1], qa[2], qa[3]] });
    World3D.update3DTransform(worldId, cubeB, { rotation: [qb[0], qb[1], qb[2], qb[3]] });
    World3D.update3DTransform(worldId, cubeC, { rotation: [qc[0], qc[1], qc[2], qc[3]] });

    if (elapsedSeconds <= 1.5 && elapsedSeconds >= nextShadowRefreshSeconds) {
      World3D.configure3DShadows(worldId, SHADOW_CONFIG);
      nextShadowRefreshSeconds += 0.25;
    }
  };
}

function startRenderLoop(update: DemoUpdate): void {
  let last = performance.now();
  let totalMs = 0;

  const frame = (now: number) => {
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    update(dtMs / 1000);
    tick(totalMs, dtMs);
    requestAnimationFrame(frame);
  };

  requestAnimationFrame(frame);
}

async function main(): Promise<void> {
  await initWasmTransport();
  initEngine({ transport: transportWasm });

  const demoId = getDemoId();
  const { worldId } = setupCommonWindow();

  const setup = demoId === '001' ? setupDemo001 : setupDemo001;
  const update = setup(worldId);
  startRenderLoop(update);
}

main().catch((error) => {
  console.error('[Galfus WASM] Fatal error:', error);
});
