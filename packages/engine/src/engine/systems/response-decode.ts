import type { System } from '../ecs';
import { enqueueCommand, markRoutingIndexDirty } from '../bridge/dispatch';
import { engineState } from '../state';

const MAX_REALM_CREATE_RETRIES = 8;
const BASE_REALM_RETRY_DELAY_MS = 32;

function flushPendingTargetLayerBinds(world: Parameters<System>[0], worldId: number): void {
  const realmId = world.coreRealmId;
  if (realmId === undefined) return;

  for (const binding of world.targetLayerBindings.values()) {
    enqueueCommand(worldId, 'cmd-target-layer-upsert', {
      realmId,
      targetId: binding.targetId,
      layout: binding.layout,
      cameraId: binding.cameraId,
      environmentId: binding.environmentId
    });
  }
}

/**
 * Decodes and applies command responses routed to a world.
 *
 * Responsibilities:
 * - track realm/window ids returned by core
 * - retry realm creation on transient host-window races
 * - complete deferred target binds after target-upsert acknowledgements
 */
export const ResponseDecodeSystem: System = (world, context) => {
  for (let i = 0; i < world.inboundResponses.length; i++) {
    const res = world.inboundResponses[i]!;
    const content = res.content as { success?: boolean; message?: string };

    if (content && typeof content.success === 'boolean' && !content.success) {
      if (
        res.type === 'realm-create' &&
        world.coreRealmId === undefined &&
        typeof content.message === 'string' &&
        content.message.includes('Host window') &&
        content.message.includes('not found')
      ) {
        if (world.realmCreateRetryCount < MAX_REALM_CREATE_RETRIES) {
          const nowMs = engineState.clock.lastTime;
          if (nowMs >= world.nextRealmCreateRetryAtMs) {
            const retryDelay = BASE_REALM_RETRY_DELAY_MS * (1 << world.realmCreateRetryCount);
            world.realmCreateRetryCount += 1;
            world.nextRealmCreateRetryAtMs = nowMs + retryDelay;
            enqueueCommand(context.worldId, 'cmd-realm-create', world.realmCreateArgs);
          }
        } else if (world.realmCreateRetryCount === MAX_REALM_CREATE_RETRIES) {
          console.error(
            `[World ${context.worldId}] realm-create retries exhausted (${MAX_REALM_CREATE_RETRIES}). Last error: ${content.message}`
          );
          world.realmCreateRetryCount += 1;
        }
        continue;
      }
      console.error(
        `[World ${context.worldId}] Command ${res.type} (ID: ${res.id}) failed: ${content.message}`
      );
    } else if (res.type === 'realm-create') {
      const created = res.content as { realmId?: number };
      if (typeof created.realmId === 'number') {
        world.coreRealmId = created.realmId;
        world.realmCreateRetryCount = 0;
        world.nextRealmCreateRetryAtMs = 0;
        markRoutingIndexDirty();
        flushPendingTargetLayerBinds(world, context.worldId);
      }
    } else if (res.type === 'window-create') {
      const created = res.content as {
        realmId?: number;
        surfaceId?: number;
        presentId?: number;
      };
      if (typeof created.realmId === 'number') {
        world.coreRealmId = created.realmId;
        world.realmCreateRetryCount = 0;
        world.nextRealmCreateRetryAtMs = 0;
        markRoutingIndexDirty();
        flushPendingTargetLayerBinds(world, context.worldId);
      }
      if (typeof created.surfaceId === 'number') {
        world.coreSurfaceId = created.surfaceId;
      }
      if (typeof created.presentId === 'number') {
        world.corePresentId = created.presentId;
      }
    }
  }
  world.inboundResponses.length = 0;
};
