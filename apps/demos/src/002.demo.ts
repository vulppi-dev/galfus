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

const RUN_DURATION_MS = 6_000;
const FRAME_TARGET_MS = 16;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus Demo 002 - Optical Persistence',
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
    clearColor: [0.01, 0.01, 0.02, 1],
    post: {
      bloomEnabled: true,
      bloomStrength: 0.35,
      bloomThreshold: 0.75,
      bloomRadius: 0.25,
      outlineEnabled: false
    }
  });

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo002Cube'
  });

  const ghostMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo002Ghost',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.1, 0.85, 1.0, 0.65],
        emissiveColor: [0.08, 0.2, 0.4, 1.0],
        renderSide: 'double-side',
        surfaceType: 'transparent'
      }
    }
  });

  const fresnelMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo002FresnelApprox',
    options: {
      type: 'standard',
      content: {
        baseColor: [0.95, 0.4, 0.2, 1.0],
        emissiveColor: [0.2, 0.05, 0.02, 1.0],
        renderSide: 'double-side',
        surfaceType: 'opaque'
      }
    }
  });

  const pbrMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'pbr',
    label: 'Demo002Pbr',
    options: {
      type: 'pbr',
      content: {
        baseColor: [0.3, 0.45, 1.0, 1.0],
        metallic: 0.35,
        roughness: 0.25,
        ao: 1.0,
        normalScale: 1.0,
        renderSide: 'double-side',
        surfaceType: 'opaque'
      }
    }
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 120.0,
    order: 0
  });
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2.3, 7.5],
    rotation: [-0.1305262, 0, 0, 0.9914449]
  });

  const lightEntity = World3D.create3DEntity(worldId);
  World3D.create3DLight(worldId, lightEntity, {
    kind: 'directional',
    color: [1, 1, 1],
    intensity: 3.2,
    castShadow: true,
    direction: [-0.45, -1, -0.35]
  });
  World3D.update3DTransform(worldId, lightEntity, {
    position: [0, 4, 0]
  });

  const cubeGhost = World3D.create3DEntity(worldId);
  const cubeFresnel = World3D.create3DEntity(worldId);
  const cubePbr = World3D.create3DEntity(worldId);

  World3D.create3DModel(worldId, cubeGhost, {
    geometryId: cubeGeometryId,
    materialId: ghostMaterialId,
    castShadow: false,
    receiveShadow: false
  });
  World3D.create3DModel(worldId, cubeFresnel, {
    geometryId: cubeGeometryId,
    materialId: fresnelMaterialId,
    castShadow: true,
    receiveShadow: true
  });
  World3D.create3DModel(worldId, cubePbr, {
    geometryId: cubeGeometryId,
    materialId: pbrMaterialId,
    castShadow: true,
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

    const ySpin = t * 1.8;
    const radius = 1.8;
    const qA = quat.fromEuler(quat.create(), 0, (ySpin * 180) / Math.PI, 0);
    const qB = quat.fromEuler(quat.create(), 0, ((ySpin + 1.2) * 180) / Math.PI, 0);
    const qC = quat.fromEuler(quat.create(), 0, ((ySpin + 2.4) * 180) / Math.PI, 0);

    World3D.update3DTransform(worldId, cubeGhost, {
      position: [Math.cos(t * 1.3) * radius, 0.0, Math.sin(t * 1.3) * radius],
      rotation: [qA[0], qA[1], qA[2], qA[3]]
    });
    World3D.update3DTransform(worldId, cubeFresnel, {
      position: [Math.cos(t * 1.3 + 2.09) * radius, 0.0, Math.sin(t * 1.3 + 2.09) * radius],
      rotation: [qB[0], qB[1], qB[2], qB[3]]
    });
    World3D.update3DTransform(worldId, cubePbr, {
      position: [Math.cos(t * 1.3 + 4.18) * radius, 0.0, Math.sin(t * 1.3 + 4.18) * radius],
      rotation: [qC[0], qC[1], qC[2], qC[3]]
    });

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((resolve) => setTimeout(resolve, Math.max(0, FRAME_TARGET_MS - frameElapsed)));
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
