import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick,
  uploadBuffer
} from '@vulfram/engine';
import { transportBunFfi } from '@vulfram/transport-bun';
import { quat } from '@vulfram/engine/math';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const RUN_DURATION_MS = 5_000;
const FRAME_TARGET_MS = 16;
const TEXTURE_BUFFER_ID = 1;
const TEXTURE_PATH = fileURLToPath(new URL('../assets/color_texture.png', import.meta.url));

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Vulfram Demo 001',
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
    msaa: { enabled: true, sampleCount: 1 },
    skybox: {
      mode: 'none',
      intensity: 0,
      rotation: 0,
      groundColor: [0.02, 0.03, 0.04],
      horizonColor: [0.12, 0.16, 0.22],
      skyColor: [0.2, 0.35, 0.6],
      cubemapTextureId: null
    },
    clearColor: [0.03, 0.03, 0.04, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1.1,
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
      bloomScatter: 0.7
    }
  });

  World3D.configure3DShadows(worldId, {
    tileResolution: 512,
    atlasTilesW: 16,
    atlasTilesH: 16,
    atlasLayers: 2,
    virtualGridSize: 1,
    smoothing: 2,
    normalBias: 0.02
  });

  const textureBytes = readFileSync(TEXTURE_PATH);
  uploadBuffer(TEXTURE_BUFFER_ID, 'image-data', textureBytes);

  const textureId = World3D.create3DTexture(worldId, {
    source: { type: 'buffer', bufferId: TEXTURE_BUFFER_ID },
    srgb: true,
    mode: 'standalone',
    label: 'Test Texture'
  });

  const geometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube'
  });

  const materialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Textured Material',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        surfaceType: 'opaque',
        baseTexId: textureId,
        baseSampler: 'linear-clamp',
        flags: 0
      }
    }
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 6, 12],
    rotation: [-0.17364818, 0, 0, 0.98480775]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 100,
    order: 0
  });

  const lightEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEntity, {
    position: [0, 8, 4],
    rotation: [0, 0, 0, 1]
  });
  World3D.create3DLight(worldId, lightEntity, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 12,
    range: 30,
    castShadow: true
  });

  const cubeEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubeEntity, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cubeEntity, {
    geometryId,
    materialId,
    castShadow: true,
    receiveShadow: true
  });

  const floorEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, floorEntity, {
    position: [0, -2.5, 0],
    rotation: [0, 0, 0, 1],
    scale: [20, 0.1, 20]
  });
  World3D.create3DModel(worldId, floorEntity, {
    geometryId,
    materialId,
    castShadow: false,
    receiveShadow: true
  });

  const start = performance.now();
  let last = start;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const t = totalMs / 1000;
    const q = quat.fromEuler(quat.create(), t * 20, t * 45, 0);
    World3D.update3DTransform(worldId, cubeEntity, {
      rotation: [q[0], q[1], q[2], q[3]],
      position: [0, Math.sin(t) * 0.4, 0]
    });

    World3D.draw3DGizmoLine(worldId, {
      start: [0, 0, 0],
      end: [5, 0, 0],
      color: [1, 0, 0, 1]
    });
    World3D.draw3DGizmoLine(worldId, {
      start: [0, 0, 0],
      end: [0, 5, 0],
      color: [0, 1, 0, 1]
    });
    World3D.draw3DGizmoLine(worldId, {
      start: [0, 0, 0],
      end: [0, 0, 5],
      color: [0, 0, 1, 1]
    });
    World3D.draw3DGizmoAabb(worldId, {
      min: [-5, -5, -5],
      max: [5, 5, 5],
      color: [1, 1, 1, 0.2]
    });

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((r) => setTimeout(r, Math.max(0, FRAME_TARGET_MS - frameElapsed)));
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
