import type {
  CmdTargetMeasurementArgs,
  CmdTargetDisposeArgs,
  CmdTargetLayerDisposeArgs,
  CmdTargetLayerUpsertArgs,
  CmdTargetUpsertArgs,
  TargetLayerLayout
} from '../../../types/cmds/target';
import type {
  CmdInputTargetListenerDisposeArgs,
  CmdInputTargetListenerListArgs,
  CmdInputTargetListenerUpsertArgs
} from '../../../types/cmds/input';
import { enqueueCommand, markRoutingIndexDirty } from '../../bridge/dispatch';
import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import type { CameraComponent } from '../../ecs';
import { allocateGlobalId, recalculateWorldWindowBindings } from './common';
import { getWorldRealmId } from './world-state';

/**
 * Upserts a target used to present world output.
 */
export function upsertTarget(worldId: number, args: CmdTargetUpsertArgs): number {
  const id = enqueueCommand(worldId, 'cmd-target-upsert', args);
  const world = getWorldOrThrow(worldId);
  if (args.kind === 'window' && args.windowId !== undefined) {
    world.targetWindowBindings.set(args.targetId, args.windowId);
    recalculateWorldWindowBindings(world);
    markRoutingIndexDirty();
  } else {
    if (world.targetWindowBindings.delete(args.targetId)) {
      recalculateWorldWindowBindings(world);
    }
    markRoutingIndexDirty();
  }
  return id;
}

/**
 * Requests size measurements for a target.
 */
export function measureTarget(worldId: number, args: CmdTargetMeasurementArgs): number {
  return enqueueCommand(worldId, 'cmd-target-measurement', args);
}

/**
 * Creates or updates an input listener bound to a routed target.
 */
export function upsertInputTargetListener(
  worldId: number,
  args: CmdInputTargetListenerUpsertArgs
): number {
  return enqueueCommand(worldId, 'cmd-input-target-listener-upsert', args);
}

/**
 * Disposes an input listener bound to routed targets.
 */
export function disposeInputTargetListener(
  worldId: number,
  args: CmdInputTargetListenerDisposeArgs
): number {
  return enqueueCommand(worldId, 'cmd-input-target-listener-dispose', args);
}

/**
 * Lists input listeners (optionally filtered by target id).
 */
export function listInputTargetListeners(
  worldId: number,
  args: CmdInputTargetListenerListArgs = {}
): number {
  return enqueueCommand(worldId, 'cmd-input-target-listener-list', args);
}

/**
 * Disposes a target.
 */
export function disposeTarget(worldId: number, args: CmdTargetDisposeArgs): number {
  const world = getWorldOrThrow(worldId);
  world.targetLayerBindings.delete(args.targetId);
  if (world.targetWindowBindings.delete(args.targetId)) {
    recalculateWorldWindowBindings(world);
  }
  markRoutingIndexDirty();
  return enqueueCommand(worldId, 'cmd-target-dispose', args);
}

function findPreferredCameraId(worldId: number): number | undefined {
  const world = getWorldOrThrow(worldId);
  let bestId: number | undefined;
  let bestOrder = Number.POSITIVE_INFINITY;
  for (const store of world.components.values()) {
    const camera = store.get('Camera') as CameraComponent | undefined;
    if (!camera) continue;
    if (
      camera.order < bestOrder ||
      (camera.order === bestOrder && (bestId === undefined || camera.id < bestId))
    ) {
      bestOrder = camera.order;
      bestId = camera.id;
    }
  }
  return bestId;
}

/**
 * Binds this world's realm to a target layer.
 */
export function bindWorldToTarget(
  worldId: number,
  args: Omit<CmdTargetLayerUpsertArgs, 'realmId'>
): number {
  const world = getWorldOrThrow(worldId);

  const resolvedCameraId = args.cameraId ?? findPreferredCameraId(worldId);

  world.targetLayerBindings.set(args.targetId, {
    targetId: args.targetId,
    layout: args.layout,
    cameraId: resolvedCameraId,
    environmentId: args.environmentId
  });
  markRoutingIndexDirty();

  const realmId = getWorldRealmId(worldId);
  if (realmId === null) {
    // Realm not ready yet: keep binding cached and let response-decode flush later.
    return 0;
  }

  return enqueueCommand(worldId, 'cmd-target-layer-upsert', {
    realmId,
    ...args,
    cameraId: resolvedCameraId
  });
}

/**
 * Unbinds this world's realm from a target layer.
 */
export function unbindWorldFromTarget(
  worldId: number,
  args: Omit<CmdTargetLayerDisposeArgs, 'realmId'>
): number {
  const world = getWorldOrThrow(worldId);
  world.targetLayerBindings.delete(args.targetId);
  if (world.targetWindowBindings.delete(args.targetId)) {
    recalculateWorldWindowBindings(world);
  }
  markRoutingIndexDirty();

  const realmId = getWorldRealmId(worldId);
  if (realmId === null) {
    // Realm not ready yet: nothing to dispose in core.
    return 0;
  }

  return enqueueCommand(worldId, 'cmd-target-layer-dispose', {
    realmId,
    ...args
  });
}

/**
 * Convenience helper: presents this world in a window via target/layer bind.
 */
export function presentWorldInWindow(
  worldId: number,
  args: {
    windowId: number;
    targetId?: number;
    layout?: TargetLayerLayout;
    cameraId?: number;
    environmentId?: number;
  }
): { targetId: number; upsertCommandId: number; bindCommandId: number } {
  requireInitialized();
  const targetId = args.targetId ?? allocateGlobalId();
  const upsertCommandId = upsertTarget(worldId, {
    targetId,
    kind: 'window',
    windowId: args.windowId
  });
  const bindCommandId = bindWorldToTarget(worldId, {
    targetId,
    layout: args.layout ?? {
      left: { unit: 'percent', value: 0 },
      top: { unit: 'percent', value: 0 },
      width: { unit: 'percent', value: 100 },
      height: { unit: 'percent', value: 100 },
      zIndex: 0,
      blendMode: 0
    },
    cameraId: args.cameraId,
    environmentId: args.environmentId
  });
  return { targetId, upsertCommandId, bindCommandId };
}
