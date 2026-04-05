import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick,
  uploadBuffer,
  type EntityId,
  type World3DId,
} from '@vulfram/engine';
import { transportBunFfi } from '@vulfram/transport-bun';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const FRAME_TARGET_MS = 16;
const KeyCode = {
  Escape: 106,
  KeyA: 19,
  KeyD: 22,
  ArrowLeft: 72,
  ArrowRight: 73,
} as const;

const PADDLE_WIDTH = 0.2;
const PADDLE_HEIGHT = 1.5;
const PADDLE_DEPTH = 0.2;
const BALL_SIZE = 0.3;
const PADDLE_SPEED = 8.0;
const BALL_SPEED = 6.0;
const FIELD_WIDTH = 10;
const FIELD_HEIGHT = 8;

const AUDIO_BOUNCE_PATH = fileURLToPath(
  new URL('../assets/audio/ball_hit_01.wav', import.meta.url),
);
const AUDIO_BUFFER_ID = 1001;
const AUDIO_RESOURCE_ID = 5001;
const AUDIO_SOURCE_ID = 7001;

interface GameState {
  leftPaddleY: number;
  rightPaddleY: number;
  ballX: number;
  ballY: number;
  ballVelX: number;
  ballVelY: number;
  leftPaddleEntity: EntityId;
  rightPaddleEntity: EntityId;
  ballEntity: EntityId;
  audioEnabled: boolean;
}

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Vulfram Pong',
    size: [1024, 768],
    position: [100, 100],
  });
  let elapsedMs = 0;

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
      cubemapTextureId: null,
    },
    clearColor: [0.02, 0.02, 0.03, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1.05,
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
      bloomScatter: 0.7,
    },
  });

  World3D.configure3DShadows(worldId, {
    tileResolution: 1024,
    atlasTilesW: 8,
    atlasTilesH: 8,
    atlasLayers: 2,
    virtualGridSize: 1,
    smoothing: 2,
    normalBias: 0.01,
  });

  const bounceAudioBytes = readFileSync(AUDIO_BOUNCE_PATH);
  uploadBuffer(AUDIO_BUFFER_ID, 'binary-asset', bounceAudioBytes);
  World3D.create3DAudioResource(worldId, {
    resourceId: AUDIO_RESOURCE_ID,
    bufferId: AUDIO_BUFFER_ID,
  });

  const redTextureId = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.9, 0.2, 0.2, 1] },
    srgb: true,
    label: 'Red Texture',
  });
  const blueTextureId = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [0.2, 0.4, 0.9, 1] },
    srgb: true,
    label: 'Blue Texture',
  });
  const yellowTextureId = World3D.create3DTexture(worldId, {
    source: { type: 'color', color: [1, 0.9, 0.2, 1] },
    srgb: true,
    label: 'Yellow Texture',
  });

  const redMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Red Material',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: redTextureId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });
  const blueMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Blue Material',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: blueTextureId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });
  const yellowMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Yellow Material',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: yellowTextureId,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque',
      },
    },
  });

  const paddleGeomId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Paddle',
  });
  const ballGeomId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'sphere',
    label: 'Ball',
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 0, 12],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 100,
    order: 0,
  });
  World3D.update3DAudioListener(worldId, {
    position: [0, 0, 12],
    velocity: [0, 0, 0],
    forward: [0, 0, -1],
    up: [0, 1, 0],
  });

  const lightEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEntity, {
    position: [0, 0, 12],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
  });
  World3D.create3DLight(worldId, lightEntity, {
    kind: 'spot',
    color: [1, 1, 1],
    intensity: 11,
    range: 28,
    direction: [0, 0, -1],
    spotInnerOuter: [0.2, 0.6],
    castShadow: true,
  });

  const leftPaddleEntity = World3D.create3DEntity(worldId);
  const rightPaddleEntity = World3D.create3DEntity(worldId);
  const ballEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, leftPaddleEntity, {
    position: [-FIELD_WIDTH / 2 + 0.5, 0, 0],
    scale: [PADDLE_WIDTH, PADDLE_HEIGHT, PADDLE_DEPTH],
    rotation: [0, 0, 0, 1],
  });
  World3D.update3DTransform(worldId, rightPaddleEntity, {
    position: [FIELD_WIDTH / 2 - 0.5, 0, 0],
    scale: [PADDLE_WIDTH, PADDLE_HEIGHT, PADDLE_DEPTH],
    rotation: [0, 0, 0, 1],
  });
  World3D.update3DTransform(worldId, ballEntity, {
    position: [0, 0, 0],
    scale: [BALL_SIZE * 2, BALL_SIZE * 2, BALL_SIZE * 2],
    rotation: [0, 0, 0, 1],
  });
  World3D.create3DModel(worldId, leftPaddleEntity, {
    geometryId: paddleGeomId,
    materialId: redMaterialId,
    castShadow: true,
    receiveShadow: false,
  });
  World3D.create3DModel(worldId, rightPaddleEntity, {
    geometryId: paddleGeomId,
    materialId: blueMaterialId,
    castShadow: true,
    receiveShadow: false,
  });
  World3D.create3DModel(worldId, ballEntity, {
    geometryId: ballGeomId,
    materialId: yellowMaterialId,
    castShadow: true,
    receiveShadow: false,
  });

  for (let i = 0; i < 20; i++) {
    elapsedMs += FRAME_TARGET_MS;
    tick(elapsedMs, FRAME_TARGET_MS);
    await new Promise((r) => setTimeout(r, FRAME_TARGET_MS));
  }

  let audioEnabled = false;
  try {
    World3D.create3DAudioSource(worldId, {
      sourceId: AUDIO_SOURCE_ID,
      entityId: ballEntity,
      position: [0, 0, 0],
      velocity: [0, 0, 0],
      orientation: [0, 0, 0, 1],
      gain: 0.8,
      pitch: 1.0,
      spatial: {
        minDistance: 0.5,
        maxDistance: 30.0,
        rolloff: 1.0,
        coneInner: 360,
        coneOuter: 360,
        coneOuterGain: 0,
      },
    });
    audioEnabled = true;
  } catch {
    audioEnabled = false;
  }

  const state: GameState = {
    leftPaddleY: 0,
    rightPaddleY: 0,
    ballX: 0,
    ballY: 0,
    ballVelX: BALL_SPEED * (Math.random() > 0.5 ? 1 : -1),
    ballVelY: BALL_SPEED * (Math.random() - 0.5) * 0.5,
    leftPaddleEntity,
    rightPaddleEntity,
    ballEntity,
    audioEnabled,
  };

  const start = performance.now();
  let lastTime = start;
  while (elapsedMs < 300000) {
    const frameStart = performance.now();
    const dt = (frameStart - lastTime) / 1000;
    lastTime = frameStart;
    tick(elapsedMs, FRAME_TARGET_MS);
    if (
      World3D.is3DWindowCloseRequested(worldId) ||
      World3D.is3DKeyPressed(worldId, KeyCode.Escape)
    )
      break;
    updateGame(worldId, state, dt);
    const frameTime = performance.now() - frameStart;
    await new Promise((r) =>
      setTimeout(r, Math.max(0, FRAME_TARGET_MS - frameTime)),
    );
    elapsedMs += FRAME_TARGET_MS;
  }
  closeWindow(windowId);
  tick(elapsedMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

function updateGame(worldId: World3DId, state: GameState, dt: number) {
  const playBounce = (intensity: number) => {
    if (!state.audioEnabled) return;
    World3D.play3DAudioSource(worldId, {
      sourceId: AUDIO_SOURCE_ID,
      resourceId: AUDIO_RESOURCE_ID,
      intensity,
      mode: 'once',
    });
  };

  if (World3D.is3DKeyPressed(worldId, KeyCode.KeyA))
    state.leftPaddleY += PADDLE_SPEED * dt;
  if (World3D.is3DKeyPressed(worldId, KeyCode.KeyD))
    state.leftPaddleY -= PADDLE_SPEED * dt;
  if (World3D.is3DKeyPressed(worldId, KeyCode.ArrowLeft))
    state.rightPaddleY += PADDLE_SPEED * dt;
  if (World3D.is3DKeyPressed(worldId, KeyCode.ArrowRight))
    state.rightPaddleY -= PADDLE_SPEED * dt;

  const maxPaddleY = FIELD_HEIGHT / 2 - PADDLE_HEIGHT / 2;
  state.leftPaddleY = Math.max(
    -maxPaddleY,
    Math.min(maxPaddleY, state.leftPaddleY),
  );
  state.rightPaddleY = Math.max(
    -maxPaddleY,
    Math.min(maxPaddleY, state.rightPaddleY),
  );

  state.ballX += state.ballVelX * dt;
  state.ballY += state.ballVelY * dt;
  const maxBallY = FIELD_HEIGHT / 2 - BALL_SIZE / 2;
  if (state.ballY > maxBallY) {
    state.ballY = maxBallY;
    state.ballVelY = -Math.abs(state.ballVelY);
    playBounce(0.7);
  } else if (state.ballY < -maxBallY) {
    state.ballY = -maxBallY;
    state.ballVelY = Math.abs(state.ballVelY);
    playBounce(0.7);
  }

  const leftPaddleX = -FIELD_WIDTH / 2 + 0.5;
  const rightPaddleX = FIELD_WIDTH / 2 - 0.5;
  if (
    state.ballX - BALL_SIZE / 2 < leftPaddleX + PADDLE_WIDTH / 2 &&
    state.ballX > leftPaddleX &&
    Math.abs(state.ballY - state.leftPaddleY) <
      PADDLE_HEIGHT / 2 + BALL_SIZE / 2
  ) {
    state.ballX = leftPaddleX + PADDLE_WIDTH / 2 + BALL_SIZE / 2;
    state.ballVelX = Math.abs(state.ballVelX);
    playBounce(1);
  }
  if (
    state.ballX + BALL_SIZE / 2 > rightPaddleX - PADDLE_WIDTH / 2 &&
    state.ballX < rightPaddleX &&
    Math.abs(state.ballY - state.rightPaddleY) <
      PADDLE_HEIGHT / 2 + BALL_SIZE / 2
  ) {
    state.ballX = rightPaddleX - PADDLE_WIDTH / 2 - BALL_SIZE / 2;
    state.ballVelX = -Math.abs(state.ballVelX);
    playBounce(1);
  }
  if (state.ballX < -FIELD_WIDTH / 2 - 1 || state.ballX > FIELD_WIDTH / 2 + 1) {
    state.ballX = 0;
    state.ballY = 0;
    state.ballVelX = BALL_SPEED * (Math.random() > 0.5 ? 1 : -1);
    state.ballVelY = BALL_SPEED * (Math.random() - 0.5) * 0.5;
    playBounce(0.5);
  }

  World3D.update3DTransform(worldId, state.leftPaddleEntity, {
    position: [leftPaddleX, state.leftPaddleY, 0],
  });
  World3D.update3DTransform(worldId, state.rightPaddleEntity, {
    position: [rightPaddleX, state.rightPaddleY, 0],
  });
  World3D.update3DTransform(worldId, state.ballEntity, {
    position: [state.ballX, state.ballY, 0],
  });
}

main().catch((err) => {
  console.error('Error:', err);
  process.exit(1);
});
