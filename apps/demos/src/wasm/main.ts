import { Mount, World3D, createWindow, initEngine, tick } from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { initWasmTransport, transportWasm } from '@galfus/transport-browser';

const FRAME_MS = 16;

type DemoUpdate = (dtSeconds: number) => void;

function getDemoId(): string {
  const hash = window.location.hash.replace('#', '');
  if (hash.startsWith('demo-')) return hash.replace('demo-', '');
  return '001';
}

function setupCommonWindow(): { worldId: number } {
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

function setupDemo001(worldId: number): DemoUpdate {
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
        baseColor: [0.18, 0.2, 0.24, 1],
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

  const cubeA = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cubeA, {
    position: [-2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1]
  });
  World3D.create3DModel(worldId, cubeA, {
    geometryId: cubeGeometryId,
    materialId: standardMaterialId,
    castShadow: false,
    receiveShadow: false
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
    castShadow: false,
    receiveShadow: false
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
    castShadow: false,
    receiveShadow: false
  });

  const floor = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, floor, {
    position: [0, -1.1, 0],
    rotation: [-0.7071068, 0, 0, 0.7071068],
    scale: [8, 8, 1]
  });
  World3D.create3DModel(worldId, floor, {
    geometryId: floorGeometryId,
    materialId: floorMaterialId,
    castShadow: false,
    receiveShadow: false
  });

  let elapsedSeconds = 0;
  return (dtSeconds) => {
    elapsedSeconds += dtSeconds;

    const qa = quat.fromEuler(quat.create(), 0, elapsedSeconds * 1.8 * 57.2958, 0);
    const qb = quat.fromEuler(quat.create(), 0, (elapsedSeconds * 2.5 + 0.6) * 57.2958, 0);
    const qc = quat.fromEuler(quat.create(), 0, (elapsedSeconds * 1.4 + 1.2) * 57.2958, 0);

    World3D.update3DTransform(worldId, cubeA, { rotation: [qa[0], qa[1], qa[2], qa[3]] });
    World3D.update3DTransform(worldId, cubeB, { rotation: [qb[0], qb[1], qb[2], qb[3]] });
    World3D.update3DTransform(worldId, cubeC, { rotation: [qc[0], qc[1], qc[2], qc[3]] });
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
