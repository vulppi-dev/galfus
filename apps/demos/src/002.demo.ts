import { quat, vec3, type Vec3 } from '@galfus/engine/math';
import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick,
  updateWindow,
  type EntityId
} from '@galfus/engine';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 5_000;
const FRAME_TARGET_MS = 16;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus ECS Demo',
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
      intensity: 1,
      rotation: 0,
      groundColor: [0.02, 0.03, 0.04],
      horizonColor: [0.12, 0.16, 0.22],
      skyColor: [0.2, 0.35, 0.6],
      cubemapTextureId: null
    },
    clearColor: [0, 0, 0, 0],
    post: {
      filterEnabled: true,
      filterExposure: 1,
      filterGamma: 2.2,
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
    tileResolution: 1024,
    atlasTilesW: 16,
    atlasTilesH: 16,
    atlasLayers: 2,
    virtualGridSize: 2,
    smoothing: 2,
    normalBias: 0.05
  });

  const geoId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube'
  });
  const palette: Array<[number, number, number, number]> = [
    [0.92, 0.28, 0.22, 1],
    [0.16, 0.64, 0.98, 1],
    [0.26, 0.86, 0.5, 1],
    [0.98, 0.74, 0.2, 1]
  ];
  const texIds = palette.map((color, idx) =>
    World3D.create3DTexture(worldId, {
      source: { type: 'color', color },
      mode: 'standalone',
      srgb: true,
      label: `PaletteTex-${idx + 1}`
    })
  );
  const matIds = texIds.map((baseTexId, idx) =>
    World3D.create3DMaterial(worldId, {
      kind: 'standard',
      label: `Standard-${idx + 1}`,
      options: {
        type: 'standard',
        content: {
          baseColor: [1, 1, 1, 1],
          baseTexId,
          baseSampler: 'linear-clamp',
          flags: 0,
          surfaceType: 'opaque'
        }
      }
    })
  );

  const camEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, camEnt, {
    position: [0, 7, 12],
    rotation: [-0.25881904, 0, 0, 0.9659258]
  });
  World3D.create3DCamera(worldId, camEnt, {
    kind: 'perspective',
    near: 0.1,
    far: 100,
    order: 0
  });

  const lightEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEnt, { position: [0, 8, 0] });
  World3D.create3DLight(worldId, lightEnt, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 3.5,
    range: 22,
    castShadow: true
  });
  const fillLightEnt = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, fillLightEnt, { position: [-6, 10, 8] });
  World3D.create3DLight(worldId, fillLightEnt, {
    kind: 'directional',
    color: [0.95, 0.96, 1],
    intensity: 0.55,
    direction: [0.25, -1, -0.45],
    castShadow: false
  });
  const ambientLightEnt = World3D.create3DEntity(worldId);
  World3D.create3DLight(worldId, ambientLightEnt, {
    kind: 'ambient',
    color: [0.22, 0.22, 0.24],
    intensity: 0.45,
    castShadow: false
  });

  const cubes: Array<{
    ent: EntityId;
    base: [number, number, number];
    phase: number;
    axis: Vec3;
  }> = [];
  for (let i = 0; i < 64; i++) {
    const x = Math.random() * 10 - 5;
    const y = Math.random() * 6 - 3;
    const z = Math.random() * 10 - 5;
    const ent = World3D.create3DEntity(worldId);
    World3D.update3DTransform(worldId, ent, {
      position: [x, y, z],
      scale: [0.4, 0.4, 0.4],
      rotation: [0, 0, 0, 1]
    });
    World3D.create3DModel(worldId, ent, {
      geometryId: geoId,
      materialId: matIds[i % matIds.length],
      castShadow: true,
      receiveShadow: true
    });
    cubes.push({
      ent,
      base: [x, y, z],
      phase: Math.random() * Math.PI * 2,
      axis: vec3.fromValues(Math.random(), Math.random(), Math.random())
    });
  }

  World3D.send3DNotification(worldId, {
    level: 'info',
    title: 'Engine Started',
    message: 'World -> Realm -> Target funcionando.'
  });

  const start = performance.now();
  let last = start;
  let updatedTitle = false;
  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;
    const t = totalMs / 1000;

    if (!updatedTitle && t > 2) {
      updateWindow(windowId, { title: 'Galfus ECS Demo (Realm-bound)' });
      updatedTitle = true;
    }

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

    for (const cube of cubes) {
      const q = quat.create();
      quat.setAxisAngle(q, cube.axis, t + cube.phase);
      World3D.update3DTransform(worldId, cube.ent, {
        position: [cube.base[0], cube.base[1] + Math.sin(t + cube.phase) * 0.5, cube.base[2]],
        rotation: [q[0], q[1], q[2], q[3]]
      });
    }

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((r) => setTimeout(r, Math.max(0, FRAME_TARGET_MS - frameElapsed)));
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
