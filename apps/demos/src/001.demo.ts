import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@galfus/engine';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 50_000;
const FRAME_TARGET_MS = 16;
const SHADOW_CONFIG = {
  tileResolution: 1024,
  atlasTilesW: 16,
  atlasTilesH: 16,
  atlasLayers: 2,
  virtualGridSize: 1,
  smoothing: 2,
  normalBias: 0.05
} as const;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus Demo 001 - Minimal 3D',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    transparent: false,
    initialState: 'maximized'
  });

  let totalMs = 0;

  const worldId = World3D.create3DWorld();
  Mount.mountWorld(worldId, { target: { kind: 'window', windowId } });

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
      type: 'schema',
      content: {
        baseColor: [0.92, 0.35, 0.32, 1]
      }
    }
  });

  const pbrMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'pbr',
    label: 'Demo001Pbr',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.86, 0.62, 1],
        metallic: [0.55, 0, 0, 0],
        roughness: [0.35, 0, 0, 0],
        ao: [1, 0, 0, 0],
        normalScale: [1, 0, 0, 0]
      }
    }
  });

  const customSimpleMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001CustomSimplePlaceholder',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.45, 0.98, 1],
        emissiveColor: [0, 0, 0, 0],
        specColor: [1, 1, 1, 1],
        specPower: [64, 0, 0, 0]
      }
    }
  });
  const floorMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Floor',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.24, 0.24, 0.26, 1],
        specColor: [0.05, 0.05, 0.05, 1],
        specPower: [8, 0, 0, 0]
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
    color: [0, 1, 1],
    intensity: 4,
    range: 24,
    castShadow: true
  });

  const lightEntity2 = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEntity2, {
    position: [-4, 3, -2],
    rotation: [0, 0, 0, 1]
  });
  World3D.create3DLight(worldId, lightEntity2, {
    kind: 'point',
    color: [1, 0, 1],
    intensity: 2.4,
    range: 18,
    castShadow: false
  });

  const cube1 = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cube1, {
    position: [-2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cube1, {
    geometryId: cubeGeometryId,
    materialId: standardMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  const cube2 = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cube2, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cube2, {
    geometryId: cubeGeometryId,
    materialId: pbrMaterialId,
    castShadow: true,
    receiveShadow: true
  });

  const cube3 = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cube3, {
    position: [2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cube3, {
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

  const start = performance.now();
  let last = start;
  let nextShadowRefreshMs = 0;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const timeSeconds = totalMs / 1000;
    const angleA = timeSeconds * 1.8;
    const angleB = timeSeconds * 2.5 + 0.6;
    const angleC = timeSeconds * 1.4 + 1.2;

    const qA: [number, number, number, number] = [
      0,
      Math.sin(angleA * 0.5),
      0,
      Math.cos(angleA * 0.5)
    ];
    const qB: [number, number, number, number] = [
      0,
      Math.sin(angleB * 0.5),
      0,
      Math.cos(angleB * 0.5)
    ];
    const qC: [number, number, number, number] = [
      0,
      Math.sin(angleC * 0.5),
      0,
      Math.cos(angleC * 0.5)
    ];

    World3D.update3DTransform(worldId, cube1, { rotation: qA });
    World3D.update3DTransform(worldId, cube2, { rotation: qB });
    World3D.update3DTransform(worldId, cube3, { rotation: qC });

    // Re-apply shadow config during startup to cover late window/render init.
    if (totalMs <= 1500 && totalMs >= nextShadowRefreshMs) {
      World3D.configure3DShadows(worldId, SHADOW_CONFIG);
      nextShadowRefreshMs += 250;
    }

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
