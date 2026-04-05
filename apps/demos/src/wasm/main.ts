import {
  Mount,
  World3D,
  createWindow,
  initEngine,
  tick,
  type EntityId,
  type WindowId,
  type World3DId,
} from '@vulfram/engine';
import {
  attachCollisionAabb,
  createPointerRay,
  drawCollisionAabbGizmo,
  raycastCollisionAabb,
} from '@vulfram/engine/helpers';
import { loadGltfAsset } from '@vulfram/gltf-loader';
import { mat4, quat } from 'gl-matrix';
import { initWasmTransport, transportWasm } from '@vulfram/transport-browser';

let WORLD_ID = 0 as unknown as World3DId;
let WINDOW_ID = 0 as unknown as WindowId;
const KeyCode = {
  KeyA: 19,
  KeyD: 22,
  KeyW: 41,
  KeyS: 37,
  ArrowUp: 74,
  ArrowDown: 71,
} as const;

type DemoUpdate = (dt: number) => void;
type DemoSetup = () => DemoUpdate | Promise<DemoUpdate>;

async function fetchBinaryAsset(path: string): Promise<Uint8Array> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch asset "${path}" (${response.status})`);
  }
  const buffer = await response.arrayBuffer();
  return new Uint8Array(buffer);
}

function getDemoId(): string {
  const hash = window.location.hash.replace('#', '');
  if (hash.startsWith('demo-')) return hash.replace('demo-', '');
  return '001';
}

function setupCommonWindow(): void {
  const { windowId } = createWindow({
    title: 'Vulfram WASM Demo',
    size: [1024, 640],
    position: [0, 0],
    resizable: true,
    canvasId: 'vulfram-canvas',
  });
  WINDOW_ID = windowId;
  WORLD_ID = World3D.create3DWorld();
}

function setupCameraAndLight(): void {
  const cameraEntity = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, cameraEntity, {
    position: [0, 0, 10],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DCamera(WORLD_ID, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 100.0,
    order: 0,
  });

  const lightEntity = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, lightEntity, {
    position: [2, 4, 6],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DLight(WORLD_ID, lightEntity, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 20.0,
    range: 50.0,
  });

  World3D.configure3DEnvironment(WORLD_ID, {
    msaa: {
      enabled: false,
      sampleCount: 0,
    },
    skybox: {
      intensity: 1.0,
      mode: 'procedural',
      rotation: 0,
      groundColor: [0.02, 0.03, 0.04],
      horizonColor: [0.12, 0.16, 0.22],
      skyColor: [0.2, 0.35, 0.6],
      cubemapTextureId: 0,
    },
    clearColor: [0, 0, 0, 0],
    post: {
      filterEnabled: false,
      filterExposure: 1.0,
      filterGamma: 1.0,
      filterSaturation: 1.0,
      filterContrast: 1.0,
      filterVignette: 0.0,
      filterGrain: 0.0,
      filterChromaticAberration: 0.0,
      filterBlur: 0.0,
      filterSharpen: 0.0,
      filterTonemapMode: 1,
      outlineEnabled: false,
      outlineStrength: 0.0,
      outlineThreshold: 0.2,
      outlineWidth: 1.0,
      outlineQuality: 1.0,
      filterPosterizeSteps: 0.0,
      cellShading: false,
      ssaoEnabled: false,
      ssaoStrength: 1.0,
      ssaoRadius: 0.75,
      ssaoBias: 0.025,
      ssaoPower: 1.5,
      ssaoBlurRadius: 2.0,
      ssaoBlurDepthThreshold: 0.02,
      bloomEnabled: false,
      bloomThreshold: 1.0,
      bloomKnee: 0.5,
      bloomIntensity: 0.8,
      bloomScatter: 0.7,
    },
  });

  World3D.configure3DShadows(WORLD_ID, {
    tileResolution: 1024,
    atlasTilesW: 16,
    atlasTilesH: 16,
    atlasLayers: 2,
    virtualGridSize: 1,
    smoothing: 2,
    normalBias: 0.05,
  });
}

function createColorMaterial(
  label: string,
  color: [number, number, number, number],
): number {
  const texId = World3D.create3DTexture(WORLD_ID, {
    source: { type: 'color', color },
    srgb: true,
    label: `${label} Texture`,
  });
  return World3D.create3DMaterial(WORLD_ID, {
    kind: 'standard',
    label: `${label} Material`,
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: texId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });
}

function setupDemo001(): DemoUpdate {
  setupCameraAndLight();

  const cubeMat = createColorMaterial('White', [1, 1, 1, 1]);
  const cubeGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube',
  });
  const cubeEnt = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, cubeEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DModel(WORLD_ID, cubeEnt, {
    geometryId: cubeGeom,
    materialId: cubeMat,
    castShadow: true,
    receiveShadow: true,
  });

  let angle = 0;
  return (dt) => {
    angle += dt * 0.8;
    const q = quat.fromEuler(quat.create(), 0, angle * 57.2958, 0);
    World3D.update3DTransform(WORLD_ID, cubeEnt, {
      rotation: [q[0], q[1], q[2], q[3]],
    });
  };
}

function setupDemo002(): DemoUpdate {
  setupCameraAndLight();

  const cubeMat = createColorMaterial('Gray', [0.8, 0.8, 0.85, 1]);
  const cubeGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube',
  });

  const grid: { ent: EntityId; base: [number, number, number] }[] = [];
  const size = 4;
  for (let x = -size; x <= size; x++) {
    for (let y = -size; y <= size; y++) {
      const ent = World3D.create3DEntity(WORLD_ID);
      const pos: [number, number, number] = [x * 1.4, y * 1.2, 0];
      World3D.update3DTransform(WORLD_ID, ent, {
        position: pos,
        rotation: [0, 0, 0, 1],
        scale: [0.6, 0.6, 0.6],
      });
      World3D.create3DModel(WORLD_ID, ent, {
        geometryId: cubeGeom,
        materialId: cubeMat,
      });
      grid.push({ ent, base: pos });
    }
  }

  let t = 0;
  return (dt) => {
    t += dt;
    for (const item of grid) {
      const offset = Math.sin(t + item.base[0]) * 0.5;
      World3D.update3DTransform(WORLD_ID, item.ent, {
        position: [item.base[0], item.base[1], offset],
      });
    }
  };
}

function setupDemo003(): DemoUpdate {
  setupCameraAndLight();

  const blueMat = createColorMaterial('Blue', [0.3, 0.6, 0.9, 1]);
  const redMat = createColorMaterial('Red', [0.95, 0.2, 0.2, 1]);
  const cubeGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'cube',
    label: 'Cube',
  });
  const cubePivotEnt = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, cubePivotEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });

  const cubeBlueEnt = World3D.create3DEntity(WORLD_ID);
  World3D.set3DParent(WORLD_ID, cubeBlueEnt, cubePivotEnt);
  World3D.update3DTransform(WORLD_ID, cubeBlueEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DModel(WORLD_ID, cubeBlueEnt, {
    geometryId: cubeGeom,
    materialId: blueMat,
  });

  const cubeRedEnt = World3D.create3DEntity(WORLD_ID);
  World3D.set3DParent(WORLD_ID, cubeRedEnt, cubePivotEnt);
  World3D.update3DTransform(WORLD_ID, cubeRedEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [0.0001, 0.0001, 0.0001],
  });
  World3D.create3DModel(WORLD_ID, cubeRedEnt, {
    geometryId: cubeGeom,
    materialId: redMat,
  });
  const collider = attachCollisionAabb({
    worldId: WORLD_ID,
    modelEntityId: cubePivotEnt,
    halfExtents: [0.5, 0.5, 0.5],
    debugGizmoAabb: true,
    name: 'demo003-cube-collider',
  });

  const rot: [number, number, number] = [0, 0, 0];
  const pointerDragSpeed = 1.8;
  let isColliding = false;
  return (dt) => {
    const speed = 1.6;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyA)) rot[1] += speed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyD)) rot[1] -= speed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyW)) rot[0] += speed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyS)) rot[0] -= speed * dt;
    if (World3D.is3DPointerButtonPressed(WORLD_ID, 0)) {
      const targetDelta = World3D.get3DPointerTargetDelta(WORLD_ID);
      const delta = targetDelta ?? World3D.get3DPointerDelta(WORLD_ID);
      rot[1] += delta[0] * pointerDragSpeed * dt;
      rot[0] += delta[1] * pointerDragSpeed * dt;
    }

    const pointerRelative = World3D.get3DPointerTargetPosition(WORLD_ID);
    const pointerUv = World3D.get3DPointerTargetUv(WORLD_ID);
    const [windowWidth, windowHeight] = World3D.get3DWindowSize(WORLD_ID);
    let frameCollision = false;

    if (pointerUv) {
      const view = mat4.lookAt(mat4.create(), [0, 0, 10], [0, 0, 0], [0, 1, 0]);
      const projection = mat4.perspective(
        mat4.create(),
        (45 * Math.PI) / 180,
        windowWidth / Math.max(1, windowHeight),
        0.1,
        100,
      );
      const ray = createPointerRay({
        pointer: pointerUv,
        viewMatrix: view,
        projectionMatrix: projection,
        viewportSize: [1, 1],
      });
      if (ray) {
        frameCollision = raycastCollisionAabb(ray, collider) !== null;
      }
    }

    if (frameCollision !== isColliding) {
      isColliding = frameCollision;
      if (isColliding) {
        World3D.update3DTransform(WORLD_ID, cubeBlueEnt, {
          scale: [0.0001, 0.0001, 0.0001],
        });
        World3D.update3DTransform(WORLD_ID, cubeRedEnt, {
          scale: [1, 1, 1],
        });
      } else {
        World3D.update3DTransform(WORLD_ID, cubeBlueEnt, {
          scale: [1, 1, 1],
        });
        World3D.update3DTransform(WORLD_ID, cubeRedEnt, {
          scale: [0.0001, 0.0001, 0.0001],
        });
      }
    }
    drawCollisionAabbGizmo(collider);

    const q = quat.fromEuler(
      quat.create(),
      rot[0] * 57.2958,
      rot[1] * 57.2958,
      rot[2] * 57.2958,
    );
    World3D.update3DTransform(WORLD_ID, cubePivotEnt, {
      rotation: [q[0], q[1], q[2], q[3]],
    });
  };
}

function setupDemo004(): DemoUpdate {
  setupCameraAndLight();

  const redMat = createColorMaterial('Red', [0.9, 0.2, 0.2, 1]);
  const blueMat = createColorMaterial('Blue', [0.2, 0.4, 0.9, 1]);
  const yellowMat = createColorMaterial('Yellow', [1, 0.9, 0.2, 1]);
  const planeMat = createColorMaterial('Backdrop', [0.12, 0.12, 0.14, 1]);

  const paddleGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'cube',
    label: 'Paddle',
  });
  const ballGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'sphere',
    label: 'Ball',
  });
  const planeGeom = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'plane',
    label: 'Backdrop',
    options: { size: [24, 14, 1], subdivisions: 1 },
  });

  const planeEnt = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, planeEnt, {
    position: [0, 0, -4],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DModel(WORLD_ID, planeEnt, {
    geometryId: planeGeom,
    materialId: planeMat,
  });

  const leftEnt = World3D.create3DEntity(WORLD_ID);
  const rightEnt = World3D.create3DEntity(WORLD_ID);
  const ballEnt = World3D.create3DEntity(WORLD_ID);

  const fieldW = 10;
  const fieldH = 8;
  const paddleH = 1.6;
  const paddleW = 0.25;
  const ballSize = 0.35;

  let leftY = 0;
  let rightY = 0;
  let ballX = 0;
  let ballY = 0;
  let ballVelX = 3.6;
  let ballVelY = 2.4;

  World3D.update3DTransform(WORLD_ID, leftEnt, {
    position: [-fieldW / 2 + 0.6, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [paddleW, paddleH, 0.3],
  });
  World3D.update3DTransform(WORLD_ID, rightEnt, {
    position: [fieldW / 2 - 0.6, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [paddleW, paddleH, 0.3],
  });
  World3D.update3DTransform(WORLD_ID, ballEnt, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [ballSize, ballSize, ballSize],
  });

  World3D.create3DModel(WORLD_ID, leftEnt, {
    geometryId: paddleGeom,
    materialId: redMat,
  });
  World3D.create3DModel(WORLD_ID, rightEnt, {
    geometryId: paddleGeom,
    materialId: blueMat,
  });
  World3D.create3DModel(WORLD_ID, ballEnt, {
    geometryId: ballGeom,
    materialId: yellowMat,
  });

  return (dt) => {
    const paddleSpeed = 6.0;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyW))
      leftY += paddleSpeed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.KeyS))
      leftY -= paddleSpeed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.ArrowUp))
      rightY += paddleSpeed * dt;
    if (World3D.is3DKeyPressed(WORLD_ID, KeyCode.ArrowDown))
      rightY -= paddleSpeed * dt;

    leftY = Math.max(
      -fieldH / 2 + paddleH / 2,
      Math.min(fieldH / 2 - paddleH / 2, leftY),
    );
    rightY = Math.max(
      -fieldH / 2 + paddleH / 2,
      Math.min(fieldH / 2 - paddleH / 2, rightY),
    );

    ballX += ballVelX * dt;
    ballY += ballVelY * dt;

    if (ballY > fieldH / 2 - ballSize || ballY < -fieldH / 2 + ballSize) {
      ballVelY *= -1;
      ballY = Math.max(
        -fieldH / 2 + ballSize,
        Math.min(fieldH / 2 - ballSize, ballY),
      );
    }

    const leftX = -fieldW / 2 + 0.6;
    const rightX = fieldW / 2 - 0.6;
    const hitsLeft =
      ballX - ballSize < leftX + paddleW &&
      ballX > leftX &&
      Math.abs(ballY - leftY) < paddleH / 2 + ballSize * 0.3;
    const hitsRight =
      ballX + ballSize > rightX - paddleW &&
      ballX < rightX &&
      Math.abs(ballY - rightY) < paddleH / 2 + ballSize * 0.3;

    if (hitsLeft) {
      ballVelX = Math.abs(ballVelX);
    } else if (hitsRight) {
      ballVelX = -Math.abs(ballVelX);
    }

    if (ballX > fieldW / 2 + 2 || ballX < -fieldW / 2 - 2) {
      ballX = 0;
      ballY = 0;
      ballVelX = Math.sign(ballVelX) * 3.6;
    }

    World3D.update3DTransform(WORLD_ID, leftEnt, {
      position: [leftX, leftY, 0],
    });
    World3D.update3DTransform(WORLD_ID, rightEnt, {
      position: [rightX, rightY, 0],
    });
    World3D.update3DTransform(WORLD_ID, ballEnt, {
      position: [ballX, ballY, 0],
    });
  };
}

function setupDemo005(): DemoUpdate {
  setupCameraAndLight();

  const cubeGeometryId = World3D.create3DGeometry(WORLD_ID, {
    type: 'primitive',
    shape: 'cube',
    label: 'ConstraintCube',
  });
  const centerMat = createColorMaterial('Center', [0.25, 0.8, 0.55, 1]);
  const orbitMat = createColorMaterial('Orbit', [0.95, 0.45, 0.2, 1]);

  const centerCube = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, centerCube, {
    position: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1.2, 1.2, 1.2],
  });
  World3D.create3DModel(WORLD_ID, centerCube, {
    geometryId: cubeGeometryId,
    materialId: centerMat,
  });

  const orbitCube = World3D.create3DEntity(WORLD_ID);
  World3D.update3DTransform(WORLD_ID, orbitCube, {
    position: [4.2, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [0.8, 0.8, 0.8],
  });
  World3D.create3DModel(WORLD_ID, orbitCube, {
    geometryId: cubeGeometryId,
    materialId: orbitMat,
  });
  World3D.set3DParent(WORLD_ID, orbitCube, centerCube);

  let angle = 0;
  return (dt) => {
    angle += dt * 60;
    const q = quat.fromEuler(quat.create(), 0, angle, 0);
    World3D.update3DTransform(WORLD_ID, centerCube, {
      rotation: [q[0], q[1], q[2], q[3]],
    });
  };
}

async function setupDemo006(): Promise<DemoUpdate> {
  setupCameraAndLight();

  const glbBytes = await fetchBinaryAsset('/assets/treehouse_concept.glb');
  const asset = await loadGltfAsset({
    worldId: WORLD_ID,
    data: glbBytes,
    materialMode: 'standard',
    labelPrefix: 'wasm-demo006',
  });

  if (asset.warnings.length > 0) {
    for (const warning of asset.warnings) {
      console.warn('[WASM][Demo006][gltf-loader]', warning);
    }
  }

  const instance = asset.instantiate({
    rootTransform: {
      position: [0, -1.5, 0],
      rotation: [0, 0, 0, 1],
      scale: [1, 1, 1],
    },
  });

  let angle = 0;
  return (dt) => {
    angle += dt * 8;
    const q = quat.fromEuler(quat.create(), 0, angle, 0);
    World3D.update3DTransform(WORLD_ID, instance.rootEntityId, {
      position: [0, -1.5, 0],
      rotation: [q[0], q[1], q[2], q[3]],
      scale: [1, 1, 1],
    });
  };
}

const demos: Record<string, DemoSetup> = {
  '001': setupDemo001,
  '002': setupDemo002,
  '003': setupDemo003,
  '004': setupDemo004,
  '005': setupDemo005,
  '006': setupDemo006,
};

async function boot() {
  await initWasmTransport();
  initEngine({ transport: transportWasm });
  setupCommonWindow();

  const demoId = getDemoId();
  const pendingSetup = demos[demoId] ?? demos['001']!;
  let update: DemoUpdate | null = null;
  let setupPromise: Promise<DemoUpdate> | null = null;
  let presented = false;
  let mountAttempts = 0;
  let warmupFrames = 0;
  const warmupTarget = 30;

  window.addEventListener('hashchange', () => {
    window.location.reload();
  });

  let last = performance.now();
  function frame(now: number) {
    const delta = now - last;
    last = now;
    try {
      tick(now, delta);
    } catch (error) {
      console.error(`[Vulfram WASM] Tick failed on demo ${demoId}:`, error);
      return;
    }
    if (!presented && Mount.isWorldMountReady(WORLD_ID)) {
      Mount.mountWorld(WORLD_ID, {
        target: { kind: 'window', windowId: WINDOW_ID },
      });
      mountAttempts += 1;
      if (mountAttempts >= 3) {
        presented = true;
      }
      return requestAnimationFrame(frame);
    }
    if (!presented) {
      return requestAnimationFrame(frame);
    }
    if (warmupFrames < warmupTarget) {
      warmupFrames += 1;
    } else if (!update) {
      if (!setupPromise) {
        setupPromise = Promise.resolve(pendingSetup())
          .then((resolvedUpdate) => {
            update = resolvedUpdate;
            return resolvedUpdate;
          })
          .catch((error) => {
            console.error(
              `[Vulfram WASM] Failed to setup demo ${demoId}:`,
              error,
            );
            throw error;
          });
      }
    } else {
      update(delta / 1000);
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
}

boot().catch((err) => {
  console.error('[Vulfram WASM] Failed to boot:', err);
});
