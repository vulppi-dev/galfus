import { enqueueCommand } from '../bridge/dispatch';
import type {
  CameraComponent,
  Component,
  LightComponent,
  ModelComponent,
  System,
  TransformComponent,
} from '../ecs';
import { toQuat, toVec3 } from './utils';

const COMMAND_INTENT_TYPES = [
  'create-entity',
  'attach-tag',
  'set-parent',
  'update-transform',
  'remove-entity',
] as const;

function wouldCreateParentCycle(
  world: Parameters<System>[0],
  childEntityId: number,
  parentEntityId: number,
): boolean {
  let cursor: number | null = parentEntityId;
  const visited = new Set<number>();

  while (cursor !== null) {
    if (cursor === childEntityId) {
      return true;
    }
    if (visited.has(cursor)) {
      return true;
    }
    visited.add(cursor);

    const store = world.components.get(cursor);
    const parent = store?.get('Parent') as { parentId: number } | undefined;
    cursor = parent?.parentId ?? null;
  }

  return false;
}

/**
 * Applies structural ECS intents that must mutate local world state before
 * render-time command synthesis (entity lifecycle, tags, parent links, transforms).
 */
export const CommandIntentSystem: System = (world, context) => {
  const realmId = world.coreRealmId;
  const intents = world.intentStore.takeMany(COMMAND_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;
    if (intent.type === 'create-entity') {
      world.entities.add(intent.entityId);
      // Initialize default transform
      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      if (!store.has('Transform')) {
        store.set('Transform', {
          type: 'Transform',
          position: [0, 0, 0],
          rotation: [0, 0, 0, 1],
          scale: [1, 1, 1],
          layerMask: 0xffffffff,
          visible: true,
        });
      }
      world.constraintDirtyEntities.add(intent.entityId);
    } else if (intent.type === 'attach-tag') {
      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      store.set('Tag', {
        type: 'Tag',
        name: intent.props.name ?? '',
        labels: new Set(intent.props.labels ?? []),
      });
    } else if (intent.type === 'set-parent') {
      let store = world.components.get(intent.entityId);
      if (!store) {
        store = new Map();
        world.components.set(intent.entityId, store);
      }
      if (intent.parentId === null) {
        const oldParentId = world.constraintParentByChild.get(intent.entityId);
        if (oldParentId !== undefined) {
          const siblings = world.constraintChildrenByParent.get(oldParentId);
          siblings?.delete(intent.entityId);
          if (siblings && siblings.size === 0) {
            world.constraintChildrenByParent.delete(oldParentId);
          }
          world.constraintParentByChild.delete(intent.entityId);
        }
        store.delete('Parent');
      } else {
        if (intent.parentId === intent.entityId) {
          console.error(
            `[World ${context.worldId}] Invalid parent constraint: entity ${intent.entityId} cannot parent itself.`,
          );
          continue;
        }
        if (wouldCreateParentCycle(world, intent.entityId, intent.parentId)) {
          console.error(
            `[World ${context.worldId}] Invalid parent constraint: cycle detected for child ${intent.entityId} and parent ${intent.parentId}.`,
          );
          continue;
        }
        store.set('Parent', {
          type: 'Parent',
          parentId: intent.parentId,
        });

        const oldParentId = world.constraintParentByChild.get(intent.entityId);
        if (oldParentId !== undefined && oldParentId !== intent.parentId) {
          const siblings = world.constraintChildrenByParent.get(oldParentId);
          siblings?.delete(intent.entityId);
          if (siblings && siblings.size === 0) {
            world.constraintChildrenByParent.delete(oldParentId);
          }
        }
        world.constraintParentByChild.set(intent.entityId, intent.parentId);
        let children = world.constraintChildrenByParent.get(intent.parentId);
        if (!children) {
          children = new Set();
          world.constraintChildrenByParent.set(intent.parentId, children);
        }
        children.add(intent.entityId);
      }
      world.constraintDirtyEntities.add(intent.entityId);
    } else if (intent.type === 'update-transform') {
      const store = world.components.get(intent.entityId);
      if (!store) {
        continue;
      }
      const transform = store.get('Transform') as TransformComponent | undefined;
      if (!transform) {
        continue;
      }

      const nextProps = { ...intent.props };
      if (nextProps.position) {
        nextProps.position = toVec3(nextProps.position);
      }
      if (nextProps.rotation) {
        nextProps.rotation = toQuat(nextProps.rotation);
      }
      if (nextProps.scale) {
        nextProps.scale = toVec3(nextProps.scale);
      }
      Object.assign(transform, nextProps);
      world.constraintDirtyEntities.add(intent.entityId);
    } else if (intent.type === 'remove-entity') {
      if (realmId === undefined) continue;

      // Detach all children linked to this entity to keep hierarchy state coherent.
      const children = world.constraintChildrenByParent.get(intent.entityId);
      if (children) {
        for (const childId of children) {
          const childStore = world.components.get(childId);
          childStore?.delete('Parent');
          world.constraintParentByChild.delete(childId);
          world.constraintDirtyEntities.add(childId);
        }
        world.constraintChildrenByParent.delete(intent.entityId);
      }

      const parentId = world.constraintParentByChild.get(intent.entityId);
      if (parentId !== undefined) {
        const siblings = world.constraintChildrenByParent.get(parentId);
        siblings?.delete(intent.entityId);
        if (siblings && siblings.size === 0) {
          world.constraintChildrenByParent.delete(parentId);
        }
        world.constraintParentByChild.delete(intent.entityId);
      }

      const store = world.components.get(intent.entityId);
      if (store) {
        // Emit disposal commands for all components with IDs
        for (const [type, comp] of store) {
          if ('id' in comp) {
            if (type === 'Model') {
              const modelComp = comp as ModelComponent;
              enqueueCommand(context.worldId, 'cmd-model-dispose', {
                realmId,
                modelId: modelComp.id,
              });
            } else if (type === 'Camera') {
              const cameraComp = comp as CameraComponent;
              enqueueCommand(context.worldId, 'cmd-camera-dispose', {
                realmId,
                cameraId: cameraComp.id,
              });
            } else if (type === 'Light') {
              const lightComp = comp as LightComponent;
              enqueueCommand(context.worldId, 'cmd-light-dispose', {
                realmId,
                lightId: lightComp.id,
              });
            }
          }
        }
        world.components.delete(intent.entityId);
      }
      world.entities.delete(intent.entityId);
      world.resolvedEntityTransforms.delete(intent.entityId);
      world.sceneSyncMatrixScratch.delete(intent.entityId);
      world.constraintDirtyEntities.add(intent.entityId);
    }
  }
};
