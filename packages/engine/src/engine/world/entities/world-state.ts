import type {
  CmdSystemDiagnosticsSetArgs,
  CmdUploadBufferDiscardAllArgs,
} from '../../../types/cmds/system';
import { EngineError } from '../../errors';
import {
  enqueueGlobalCommand,
  markRoutingIndexDirty,
} from '../../bridge/dispatch';
import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import { engineState } from '../../state';

/**
 * Returns the core model ID for an entity, if available.
 */
export function getModelId(worldId: number, entityId: number): number | null {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const store = world.components.get(entityId);
  if (!store) return null;
  const model = store.get('Model') as { id: number } | undefined;
  return model?.id ?? null;
}

/**
 * Returns the core realm id associated with this world, if already created.
 */
export function getWorldRealmId(worldId: number): number | null {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  return world.coreRealmId ?? null;
}

/**
 * Returns true when the world has a resolved core realm id.
 */
export function isWorldReady(worldId: number): boolean {
  return getWorldRealmId(worldId) !== null;
}

/**
 * Disposes a world and optionally releases its bound realm/targets.
 * This operation is immediate on host state; core-side disposal commands are queued globally.
 */
export function disposeWorld(
  worldId: number,
  opts: {
    disposeRealm?: boolean;
    disposeTargets?: boolean;
    warnOnUndisposedResources?: boolean;
    strictResourceLifecycle?: boolean;
  } = {},
): void {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const disposeRealm = opts.disposeRealm ?? true;
  const disposeTargets = opts.disposeTargets ?? true;
  const strictResourceLifecycle = opts.strictResourceLifecycle ?? false;
  const warnOnUndisposedResources = opts.warnOnUndisposedResources ?? true;

  let retainedCoreObjectCount = 0;
  for (const store of world.components.values()) {
    for (const comp of store.values()) {
      if ('id' in comp && typeof comp.id === 'number') {
        retainedCoreObjectCount++;
      }
    }
  }
  const hasRetainedTargets = world.targetLayerBindings.size > 0;
  const hasPendingWork =
    world.intentStore.size() > 0 || world.pendingCommands.length > 0;

  if (
    (!disposeRealm && (retainedCoreObjectCount > 0 || hasPendingWork)) ||
    (!disposeTargets && hasRetainedTargets)
  ) {
    const message =
      `disposeWorld(${worldId}) called without fully releasing resources. ` +
      `retainRealm=${!disposeRealm} retainTargets=${!disposeTargets} ` +
      `trackedCoreObjects=${retainedCoreObjectCount} ` +
      `targetBindings=${world.targetLayerBindings.size} ` +
      `pendingIntents=${world.intentStore.size()} ` +
      `pendingCommands=${world.pendingCommands.length}`;
    if (strictResourceLifecycle) {
      throw new EngineError('WorldDisposeLifecycleRisk', message);
    }
    if (warnOnUndisposedResources) {
      console.warn(`[World ${worldId}] ${message}`);
    }
  }

  if (disposeTargets) {
    for (const targetId of world.targetLayerBindings.keys()) {
      let isShared = false;
      for (const [otherWorldId, otherWorld] of engineState.worlds) {
        if (otherWorldId === worldId) continue;
        if (otherWorld.targetLayerBindings.has(targetId)) {
          isShared = true;
          break;
        }
      }
      if (!isShared) {
        enqueueGlobalCommand('cmd-target-dispose', { targetId });
      }
    }
  }

  if (disposeRealm && world.coreRealmId !== undefined) {
    enqueueGlobalCommand('cmd-realm-dispose', { realmId: world.coreRealmId });
  }

  for (const [cmdId, trackedWorldId] of engineState.commandTracker) {
    if (trackedWorldId === worldId) {
      engineState.commandTracker.delete(cmdId);
    }
  }

  engineState.worlds.delete(worldId);
  markRoutingIndexDirty();
}

/**
 * Configures global runtime diagnostics and pointer tracing.
 */
export function setSystemDiagnostics(
  args: CmdSystemDiagnosticsSetArgs,
): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-system-diagnostics-set', args);
}

/**
 * Requests the core to discard all pending upload buffers.
 */
export function discardAllUploadBuffers(
  args: CmdUploadBufferDiscardAllArgs = {},
): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-upload-buffer-discard-all', args);
}
