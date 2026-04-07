import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@vulfram/engine';
import { loadGltfAsset } from '@vulfram/gltf-loader';
import { transportBunFfi } from '@vulfram/transport-bun';
import { quat } from 'gl-matrix';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const RUN_DURATION_MS = 12_000;
const FRAME_TARGET_MS = 16;
const GLB_PATH = fileURLToPath(new URL('../assets/treehouse_concept.glb', import.meta.url));

async function main() {
  initEngine({ transport: transportBunFfi });

  const { windowId } = createWindow({
    title: 'Vulfram Demo 006 - GLTF Loader',
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
      horizonColor: [0.1, 0.12, 0.16],
      skyColor: [0.18, 0.24, 0.32],
      cubemapTextureId: null
    },
    clearColor: [0.02, 0.02, 0.03, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1.0,
      filterGamma: 2.0,
      filterSaturation: 1.0,
      filterContrast: 1.0,
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

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2.6, 6.2],
    rotation: [-0.15643447, 0, 0, 0.98768836]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 500,
    order: 0
  });

  const keyLight = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, keyLight, {
    position: [8, 12, 6]
  });
  World3D.create3DLight(worldId, keyLight, {
    kind: 'point',
    color: [1, 0.98, 0.92],
    intensity: 14,
    range: 80,
    castShadow: false
  });

  const fillLight = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, fillLight, {
    position: [-6, 7, -5]
  });
  World3D.create3DLight(worldId, fillLight, {
    kind: 'point',
    color: [0.65, 0.72, 1.0],
    intensity: 8,
    range: 60,
    castShadow: false
  });

  const glbBytes = readFileSync(GLB_PATH);
  const asset = await loadGltfAsset({
    worldId,
    data: glbBytes,
    materialMode: 'standard',
    labelPrefix: 'demo006'
  });

  if (asset.warnings.length > 0) {
    for (const warning of asset.warnings) {
      console.warn('[Demo006][gltf-loader]', warning);
    }
  }

  const instance = asset.instantiate({
    rootTransform: {
      position: [0, -1.5, 0],
      rotation: [0, 0, 0, 1],
      scale: [1, 1, 1]
    }
  });

  const rootEntityId = instance.rootEntityId;

  const start = performance.now();
  let last = start;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const t = totalMs / 1000;
    const q = quat.fromEuler(quat.create(), 0, t * 8, 0);
    World3D.update3DTransform(worldId, rootEntityId, {
      position: [0, -1.5, 0],
      rotation: [q[0], q[1], q[2], q[3]],
      scale: [1, 1, 1]
    });

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((r) => setTimeout(r, Math.max(0, FRAME_TARGET_MS - frameElapsed)));
  }

  instance.disposeEntities();
  asset.disposeAll();
  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
