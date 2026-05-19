import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 10_000;
const FRAME_TARGET_MS = 16;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus Constraint Demo 005 - Parent Orbit',
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
      groundColor: [0.02, 0.02, 0.03],
      horizonColor: [0.12, 0.14, 0.2],
      skyColor: [0.2, 0.32, 0.52],
      cubemapTextureId: null
    },
    clearColor: [0.02, 0.02, 0.03, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1,
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

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2.5, 10.5],
    rotation: [-0.10452846, 0, 0, 0.9945219]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 100,
    order: 0
  });

  const keyLight = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, keyLight, { position: [4, 6, 6] });
  World3D.create3DLight(worldId, keyLight, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 10,
    range: 30,
    castShadow: false
  });
  const fillLight = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, fillLight, { position: [-6, 4, 2] });
  World3D.create3DLight(worldId, fillLight, {
    kind: 'point',
    color: [0.6, 0.7, 1.0],
    intensity: 6,
    range: 26,
    castShadow: false
  });

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'ConstraintCube'
  });
  const centerTex = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.25, 0.8, 0.55, 1] },
    srgb: true,
    label: 'CenterTex'
  });
  const orbitTex = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.95, 0.45, 0.2, 1] },
    srgb: true,
    label: 'OrbitTex'
  });
  const centerMat = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'CenterMat',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: centerTex,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque'
      }
    }
  });
  const orbitMat = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'OrbitMat',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: orbitTex,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque'
      }
    }
  });

  const centerCube = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, centerCube, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1.2, 1.2, 1.2]
  });
  World3D.create3DModel(worldId, centerCube, {
    geometryId: cubeGeometryId,
    materialId: centerMat,
    castShadow: false,
    receiveShadow: false
  });

  const orbitCube = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, orbitCube, {
    position: [4.2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [0.8, 0.8, 0.8]
  });
  World3D.create3DModel(worldId, orbitCube, {
    geometryId: cubeGeometryId,
    materialId: orbitMat,
    castShadow: false,
    receiveShadow: false
  });

  // Parent constraint: orbitCube now depends on centerCube transform.
  World3D.set3DParent(worldId, orbitCube, centerCube);

  const start = performance.now();
  let last = start;
  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const t = totalMs / 1000;
    const q = quat.fromEuler(quat.create(), 0, t * 60, 0);
    World3D.update3DTransform(worldId, centerCube, {
      rotation: [q[0], q[1], q[2], q[3]]
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
