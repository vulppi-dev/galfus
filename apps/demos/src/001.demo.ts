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

const RUN_DURATION_MS = 5_000;
const FRAME_TARGET_MS = 16;

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

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo001Cube'
  });

  const standardMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Standard',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.9, 0.3, 0.3, 1],
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
        baseColor: [0.3, 0.9, 0.35, 1],
        metallic: 0.6,
        roughness: 0.35,
        ao: 1,
        normalScale: 1,
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
        baseColor: [0.3, 0.45, 1, 1],
        emissiveColor: [0.06, 0.1, 0.22, 1],
        specColor: [1, 1, 1, 1],
        specPower: 48,
        surfaceType: 'opaque',
        flags: 0
      }
    }
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2.5, 8],
    rotation: [-0.1305262, 0, 0, 0.9914449]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 100,
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
    intensity: 10,
    range: 30,
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
    castShadow: false,
    receiveShadow: false
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
    castShadow: false,
    receiveShadow: false
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
    castShadow: false,
    receiveShadow: false
  });

  const start = performance.now();
  let last = start;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    tick(totalMs, dtMs);

    const frameElapsed = performance.now() - now;
    await new Promise((resolve) => setTimeout(resolve, Math.max(0, FRAME_TARGET_MS - frameElapsed)));
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
