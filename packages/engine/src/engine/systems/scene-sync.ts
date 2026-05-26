import { mat4, vec2, vec3, vec4 } from '../../math/index';
import type { Mat4, Vec2, Vec4 } from '../../math/index';
import type { CameraKind, LightKind } from '../../types/kinds';
import { enqueueCommand } from '../bridge/dispatch';
import type {
  CameraComponent,
  Component,
  LightComponent,
  ModelComponent,
  Shape2DComponent,
  Sprite2DComponent,
  System
} from '../ecs';
import { getResolvedEntityTransformMatrix, toVec2, toVec3, toVec4 } from './utils';

const SCENE_SYNC_INTENT_TYPES = [
  'attach-model',
  'attach-sprite2d',
  'attach-shape2d',
  'attach-camera',
  'attach-light',
  'detach-component',
  'gizmo-draw-line',
  'gizmo-draw-aabb',
  'gizmo-draw-polyline'
] as const;

function copyMatrixToScratch(
  world: Parameters<System>[0],
  entityId: number,
  matrix: ArrayLike<number>
): Mat4 {
  let scratch = world.sceneSyncMatrixScratch.get(entityId) as Mat4 | undefined;
  if (!scratch) {
    const nextScratch = new Array<number>(16) as number[];
    world.sceneSyncMatrixScratch.set(entityId, nextScratch);
    scratch = nextScratch as Mat4;
  }
  for (let i = 0; i < 16; i++) {
    scratch[i] = matrix[i] ?? 0;
  }
  return scratch as Mat4;
}

function hasNonZeroTranslation(value: ArrayLike<number>): boolean {
  return value[0] !== 0 || value[1] !== 0 || value[2] !== 0;
}

/**
 * Synchronizes ECS scene state with core scene objects.
 *
 * This system consumes attach/detach/gizmo intents and emits upsert/dispose
 * commands. It also pushes transform updates for entities whose resolved
 * constraint matrix changed in the current tick.
 */
export const SceneSyncSystem: System = (world, context) => {
  const realmId = world.coreRealmId;
  if (realmId === undefined) return;
  const intents = world.intentStore.takeMany(SCENE_SYNC_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;

    if (intent.type === 'attach-model') {
      const modelId = world.nextCoreId++;
      const transform = getResolvedEntityTransformMatrix(world, intent.entityId);
      const castOutline = intent.props.castOutline ?? false;
      const outlineColor = intent.props.outlineColor ?? vec4.create();

      enqueueCommand(context.worldId, 'cmd-model3d-upsert', {
        realmId,
        modelId,
        geometryId: intent.props.geometryId,
        materialId: intent.props.materialId,
        transform: copyMatrixToScratch(world, intent.entityId, transform),
        castShadow: intent.props.castShadow,
        receiveShadow: intent.props.receiveShadow,
        castOutline: intent.props.castOutline,
        outlineColor: intent.props.outlineColor
      });

      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      store.set('Model', {
        type: 'Model',
        id: modelId,
        geometryId: intent.props.geometryId,
        materialId: intent.props.materialId,
        castShadow: intent.props.castShadow ?? true,
        receiveShadow: intent.props.receiveShadow ?? true,
        castOutline,
        outlineColor: toVec4(outlineColor),
        skipUpdate: true
      });
    } else if (intent.type === 'attach-camera') {
      const cameraId = world.nextCoreId++;
      const transform = getResolvedEntityTransformMatrix(world, intent.entityId);
      const is2D = world.realmKind === 'two-d';
      const cameraKind = intent.props.kind ?? (is2D ? 'orthographic' : 'perspective');

      if (is2D) {
        enqueueCommand(context.worldId, 'cmd-camera2d-upsert', {
          realmId,
          cameraId,
          nearFar: vec2.fromValues(intent.props.near ?? 0.1, intent.props.far ?? 1000),
          layerMask: intent.props.layerMask ?? 0xffffffff,
          order: intent.props.order,
          transform: copyMatrixToScratch(world, intent.entityId, transform),
          orthoScale: intent.props.orthoScale ?? 1.0
        });
      } else {
        enqueueCommand(context.worldId, 'cmd-camera3d-upsert', {
          realmId,
          cameraId,
          kind: cameraKind as CameraKind,
          nearFar: vec2.fromValues(intent.props.near ?? 0.1, intent.props.far ?? 1000),
          layerMask: intent.props.layerMask ?? 0xffffffff,
          order: intent.props.order,
          transform: copyMatrixToScratch(world, intent.entityId, transform),
          orthoScale: intent.props.orthoScale,
          viewPosition: intent.props.viewPosition
        });
      }

      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      store.set('Camera', {
        type: 'Camera',
        id: cameraId,
        kind: cameraKind as CameraKind,
        near: intent.props.near ?? 0.1,
        far: intent.props.far ?? 1000,
        order: intent.props.order ?? 0,
        layerMask: intent.props.layerMask ?? 0xffffffff,
        orthoScale: intent.props.orthoScale ?? (is2D ? 1.0 : 10.0),
        skipUpdate: true
      });

      // Auto-attach the first available camera to realm target layers that were bound
      // before a camera existed (common when presenting world before scene setup).
      for (const binding of world.targetLayerBindings.values()) {
        if (binding.cameraId !== undefined) continue;
        enqueueCommand(context.worldId, 'cmd-target-layer-upsert', {
          realmId,
          targetId: binding.targetId,
          layout: binding.layout,
          cameraId,
          enabledCameraIds: [cameraId],
          environmentId: binding.environmentId
        });
        binding.cameraId = cameraId;
      }
    } else if (intent.type === 'attach-light') {
      const lightId = world.nextCoreId++;
      const transform = getResolvedEntityTransformMatrix(world, intent.entityId);

      const pos = vec3.create();
      mat4.getTranslation(pos, transform);
      const lightCmd: {
        realmId: number;
        lightId: number;
        kind?: LightKind;
        color?: Vec4;
        intensity?: number;
        range?: number;
        castShadow?: boolean;
        position?: Vec4;
        direction?: Vec4;
        spotInnerOuter?: Vec2;
      } = {
        realmId,
        lightId
      };

      if (intent.props.kind !== undefined) {
        lightCmd.kind = intent.props.kind;
      }
      if (intent.props.color !== undefined) {
        const color = toVec3(intent.props.color);
        lightCmd.color = vec4.fromValues(color[0], color[1], color[2], 1);
      }
      if (intent.props.intensity !== undefined) {
        lightCmd.intensity = intent.props.intensity;
      }
      if (intent.props.range !== undefined) {
        lightCmd.range = intent.props.range;
      }
      if (intent.props.castShadow !== undefined) {
        lightCmd.castShadow = intent.props.castShadow;
      }
      if (hasNonZeroTranslation(pos)) {
        lightCmd.position = vec4.fromValues(pos[0], pos[1], pos[2], 1);
      }
      if (intent.props.direction !== undefined) {
        const direction = toVec3(intent.props.direction);
        lightCmd.direction = vec4.fromValues(
          direction[0] ?? 0,
          direction[1] ?? 0,
          direction[2] ?? 0,
          0
        );
      }
      if (intent.props.spotInnerOuter) {
        lightCmd.spotInnerOuter = toVec2(intent.props.spotInnerOuter);
      }
      enqueueCommand(context.worldId, 'cmd-light3d-upsert', lightCmd);

      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      store.set('Light', {
        type: 'Light',
        id: lightId,
        kind: intent.props.kind ?? ('point' as LightKind),
        color: intent.props.color ? toVec3(intent.props.color) : vec3.fromValues(1, 1, 1),
        intensity: intent.props.intensity ?? 1.0,
        range: intent.props.range ?? 10.0,
        castShadow: intent.props.castShadow ?? true,
        direction: intent.props.direction
          ? toVec3(intent.props.direction)
          : vec3.fromValues(0, -1, 0),
        spotInnerOuter: intent.props.spotInnerOuter
          ? toVec2(intent.props.spotInnerOuter)
          : vec2.fromValues(0.5, 0.8),
        skipUpdate: true
      });
    } else if (intent.type === 'attach-sprite2d' || intent.type === 'attach-shape2d') {
      const objectId = world.nextCoreId++;
      const transform = getResolvedEntityTransformMatrix(world, intent.entityId);
      const layer = intent.props.layer ?? 0;
      const transformArray = copyMatrixToScratch(world, intent.entityId, transform);

      if (intent.type === 'attach-sprite2d') {
        enqueueCommand(context.worldId, 'cmd-sprite2d-upsert', {
          realmId,
          spriteId: objectId,
          geometryId: intent.props.geometryId,
          materialId: intent.props.materialId,
          transform: transformArray,
          layer
        });
      } else {
        enqueueCommand(context.worldId, 'cmd-shape2d-upsert', {
          realmId,
          shapeId: objectId,
          geometryId: intent.props.geometryId,
          materialId: intent.props.materialId,
          transform: transformArray,
          layer
        });
      }

      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      if (intent.type === 'attach-sprite2d') {
        store.set('Sprite2D', {
          type: 'Sprite2D',
          id: objectId,
          geometryId: intent.props.geometryId,
          materialId: intent.props.materialId,
          layer,
          skipUpdate: true
        });
      } else {
        store.set('Shape2D', {
          type: 'Shape2D',
          id: objectId,
          geometryId: intent.props.geometryId,
          materialId: intent.props.materialId,
          layer,
          skipUpdate: true
        });
      }
    } else if (intent.type === 'detach-component') {
      const store = world.components.get(intent.entityId);
      if (store) {
        const comp = store.get(intent.componentType) as Component | undefined;
        if (comp && 'id' in comp) {
          if (intent.componentType === 'Model') {
            const modelComp = comp as ModelComponent;
            enqueueCommand(context.worldId, 'cmd-model3d-dispose', {
              realmId,
              modelId: modelComp.id
            });
          } else if (intent.componentType === 'Camera') {
            const cameraComp = comp as CameraComponent;
            enqueueCommand(context.worldId, 'cmd-camera3d-dispose', {
              realmId,
              cameraId: cameraComp.id
            });
          } else if (intent.componentType === 'Light') {
            const lightComp = comp as LightComponent;
            enqueueCommand(context.worldId, 'cmd-light3d-dispose', {
              realmId,
              lightId: lightComp.id
            });
          } else if (intent.componentType === 'Sprite2D') {
            const spriteComp = comp as Sprite2DComponent;
            enqueueCommand(context.worldId, 'cmd-sprite2d-dispose', {
              realmId,
              spriteId: spriteComp.id
            });
          } else if (intent.componentType === 'Shape2D') {
            const shapeComp = comp as Shape2DComponent;
            enqueueCommand(context.worldId, 'cmd-shape2d-dispose', {
              realmId,
              shapeId: shapeComp.id
            });
          }
        }
        store.delete(intent.componentType);
      }
    } else if (intent.type === 'gizmo-draw-line') {
      enqueueCommand(context.worldId, 'cmd-gizmo-draw-line', {
        start: toVec3(intent.start),
        end: toVec3(intent.end),
        color: toVec4(intent.color),
        thickness: intent.thickness
      });
    } else if (intent.type === 'gizmo-draw-aabb') {
      enqueueCommand(context.worldId, 'cmd-gizmo-draw-aabb', {
        min: toVec3(intent.min),
        max: toVec3(intent.max),
        color: toVec4(intent.color),
        thickness: intent.thickness
      });
    } else if (intent.type === 'gizmo-draw-polyline') {
      enqueueCommand(context.worldId, 'cmd-gizmo-draw-polyline', {
        points: intent.points.map((point) => toVec3(point)),
        color: toVec4(intent.color),
        closed: intent.closed,
        thickness: intent.thickness
      });
    }
  }

  for (const entityId of world.constraintChangedEntities) {
    const store = world.components.get(entityId);
    if (!store) continue;

    const matrix = getResolvedEntityTransformMatrix(world, entityId);
    let matrixArray: Mat4 | undefined;

    const model = store.get('Model') as ModelComponent | undefined;
    if (model) {
      if (model.skipUpdate) {
        model.skipUpdate = false;
      } else {
        matrixArray = matrixArray ?? copyMatrixToScratch(world, entityId, matrix);
        enqueueCommand(context.worldId, 'cmd-model3d-upsert', {
          realmId,
          modelId: model.id,
          transform: matrixArray
        });
      }
    }

    const camera = store.get('Camera') as CameraComponent | undefined;
    if (camera) {
      if (camera.skipUpdate) {
        camera.skipUpdate = false;
      } else {
        matrixArray = matrixArray ?? copyMatrixToScratch(world, entityId, matrix);
        if (world.realmKind === 'two-d') {
          enqueueCommand(context.worldId, 'cmd-camera2d-upsert', {
            realmId,
            cameraId: camera.id,
            transform: matrixArray
          });
        } else {
          enqueueCommand(context.worldId, 'cmd-camera3d-upsert', {
            realmId,
            cameraId: camera.id,
            transform: matrixArray
          });
        }
      }
    }

    const light = store.get('Light') as LightComponent | undefined;
    if (light) {
      if (light.skipUpdate) {
        light.skipUpdate = false;
      } else {
        const pos = vec3.create();
        mat4.getTranslation(pos, matrix);
        enqueueCommand(context.worldId, 'cmd-light3d-upsert', {
          realmId,
          lightId: light.id,
          position: vec4.fromValues(pos[0], pos[1], pos[2], 1)
        });
      }
    }

    const sprite2d = store.get('Sprite2D') as Sprite2DComponent | undefined;
    if (sprite2d) {
      if (sprite2d.skipUpdate) {
        sprite2d.skipUpdate = false;
      } else {
        matrixArray = matrixArray ?? copyMatrixToScratch(world, entityId, matrix);
        enqueueCommand(context.worldId, 'cmd-sprite2d-upsert', {
          realmId,
          spriteId: sprite2d.id,
          transform: matrixArray
        });
      }
    }

    const shape2d = store.get('Shape2D') as Shape2DComponent | undefined;
    if (shape2d) {
      if (shape2d.skipUpdate) {
        shape2d.skipUpdate = false;
      } else {
        matrixArray = matrixArray ?? copyMatrixToScratch(world, entityId, matrix);
        enqueueCommand(context.worldId, 'cmd-shape2d-upsert', {
          realmId,
          shapeId: shape2d.id,
          transform: matrixArray
        });
      }
    }
  }
};

/** Backward-compatible alias while migrating existing integrations. */
export const CoreCommandBuilderSystem = SceneSyncSystem;
