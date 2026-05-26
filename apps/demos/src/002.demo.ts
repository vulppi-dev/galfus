import {
  Mount,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@galfus/engine';
import type { CmdMaterialDefinitionCreateArgs, CmdMaterialInstanceCreateArgs } from '@galfus/engine/types';
import { quat } from '@galfus/engine/math';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 8_000;
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
    clearColor: [0, 0, 0, 1],
    post: {
      outlineEnabled: false
    }
  });

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo002Cube'
  });
  const floorGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo002Floor'
  });

  const ghostDefinitionId = 210;
  const fresnelDefinitionId = 211;
  const ghostMaterialId = 212;
  const fresnelMaterialId = 213;
  const floorMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo002Floor',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.08, 0.08, 0.1, 1.0],
        specColor: [0.02, 0.02, 0.02, 1.0],
        specPower: [6.0, 0.0, 0.0, 0.0]
      }
    }
  });

  World3D.upsert3DMaterialDefinition(worldId, {
    definitionId: ghostDefinitionId,
    slug: 'demo2-ghost',
    label: 'demo2-ghost-definition',
    realmKind: 'three-d',
    shaderType: 'model',
    shaderSource: `
fn project_world_to_screen_uv(world_position: vec3<f32>) -> vec2<f32> {
  let clip = camera.view_projection * vec4<f32>(world_position, 1.0);
  let inv_w = select(0.0, 1.0 / clip.w, abs(clip.w) > 1e-6);
  let ndc = clip.xy * inv_w;
  return vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
}

fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let base = vec3<f32>(0.08, 0.48, 0.88);
  let screen_uv = project_world_to_screen_uv(input.world_position);
  let ghost_uv = screen_uv + vec2<f32>(0.012, 0.0);
  let history = sample_history0(ghost_uv).rgb;
  let trail_decay = 0.965;
  let history_weight = 0.62;
  let persisted = history * trail_decay;
  let ghost_tint = vec3<f32>(0.18, 0.9, 1.0);
  let ghost = persisted * ghost_tint * history_weight;
  let composed = max(base, base + ghost);

  out.color = vec4<f32>(composed, 1.0);
  out.emissive = vec4<f32>(ghost_tint * 0.08, 1.0);
  return out;
}
`,
    capabilities: { semantics: ['history0'] },
    shaderParamsSchema: {}
  } satisfies CmdMaterialDefinitionCreateArgs);

  World3D.upsert3DMaterialDefinition(worldId, {
    definitionId: fresnelDefinitionId,
    slug: 'demo2-fresnel',
    label: 'demo2-fresnel-definition',
    realmKind: 'three-d',
    shaderType: 'model',
    shaderSource: `
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let t = frame.time;
  let base = vec3<f32>(0.95, 0.28, 0.18);
  let view_dir = normalize(camera.position.xyz - input.world_position);
  let fresnel = pow(1.0 - max(dot(normalize(input.world_normal), view_dir), 0.0), 2.4);
  let pulse = 0.5 + 0.5 * sin(t * 5.0);
  let rim = fresnel * (0.35 + 0.45 * pulse);
  let color = mix(base, vec3<f32>(1.0, 0.92, 0.82), rim);
  out.color = vec4<f32>(color, 1.0);
  out.emissive = vec4<f32>(vec3<f32>(1.0, 0.45, 0.28) * (0.08 + 0.55 * rim), 1.0);
  return out;
}
`,
    shaderParamsSchema: {}
  } satisfies CmdMaterialDefinitionCreateArgs);

  World3D.upsert3DMaterialInstance(worldId, {
    materialId: ghostMaterialId,
    slug: 'demo2-ghost',
    label: 'demo2-mat-ghost',
    options: { type: 'schema', content: { baseColor: [0.2, 0.7, 1.0, 1.0] } }
  } satisfies CmdMaterialInstanceCreateArgs);
  World3D.upsert3DMaterialInstance(worldId, {
    materialId: fresnelMaterialId,
    slug: 'demo2-fresnel',
    label: 'demo2-mat-fresnel',
    options: { type: 'schema', content: { baseColor: [1.0, 0.3, 0.2, 1.0] } }
  } satisfies CmdMaterialInstanceCreateArgs);

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
    kind: 'point',
    color: [1, 1, 1],
    intensity: 3.5,
    range: 24,
    castShadow: true,
  });
  World3D.update3DTransform(worldId, lightEntity, {
    position: [3, 4, 4]
  });

  const cubeGhost = World3D.create3DEntity(worldId);
  const cubeFresnel = World3D.create3DEntity(worldId);
  const floor = World3D.create3DEntity(worldId);

  World3D.create3DModel(worldId, cubeGhost, {
    geometryId: cubeGeometryId,
    materialId: ghostMaterialId,
    castShadow: true,
    receiveShadow: true
  });
  World3D.create3DModel(worldId, cubeFresnel, {
    geometryId: cubeGeometryId,
    materialId: fresnelMaterialId,
    castShadow: true,
    receiveShadow: true
  });
  World3D.update3DTransform(worldId, floor, {
    position: [0.0, -0.15, 0.0],
    rotation: [-0.7071068, 0, 0, 0.7071068],
    scale: [12, 12, 1]
  });
  World3D.create3DModel(worldId, floor, {
    geometryId: floorGeometryId,
    materialId: floorMaterialId,
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

    const qA = quat.fromEuler(quat.create(), (t * 1.7 * 180) / Math.PI, (t * 2.8 * 180) / Math.PI, 0);
    const qB = quat.fromEuler(quat.create(), (t * 1.3 * 180) / Math.PI, (-(t * 2.2) * 180) / Math.PI, 0);

    World3D.update3DTransform(worldId, cubeGhost, {
      position: [-2.0 + Math.sin(t * 2.4) * 1.8, 0.8 + Math.sin(t * 3.0) * 0.28, 0.0],
      rotation: [qA[0], qA[1], qA[2], qA[3]]
    });
    World3D.update3DTransform(worldId, cubeFresnel, {
      position: [2.0 + Math.sin(t * 2.0) * 1.2, 0.8 + Math.sin(t * 2.6) * 0.2, 0.0],
      rotation: [qB[0], qB[1], qB[2], qB[3]]
    });

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
