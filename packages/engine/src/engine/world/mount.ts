import type {
  CmdTargetMeasurementArgs,
  CmdTargetLayerDisposeArgs,
  CmdTargetLayerUpsertArgs,
  CmdTargetUpsertArgs,
  TargetLayerLayout
} from '../../types/cmds/target';
import { engineState } from '../state';
import {
  bindWorldToTarget,
  isWorldReady,
  measureTarget,
  unbindWorldFromTarget,
  upsertTarget
} from './entities';
import type { CommandId, TargetId, WindowId, WorldId } from './types';
import { asCommandId, asTargetId, asWorldNumber } from './types';

type MountTargetConfig = Omit<CmdTargetUpsertArgs, 'targetId'>;

export type MountWorldArgs = {
  target: MountTargetConfig;
  targetId?: TargetId;
  layout?: TargetLayerLayout;
  cameraId?: number;
  environmentId?: number;
};

export type WaitWorldReadyOptions = {
  timeoutMs?: number;
  pollIntervalMs?: number;
  onPoll?: (() => void | Promise<void>) | undefined;
};

/**
 * Returns whether the world has resolved its internal core realm and can be mounted.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * const ready = Mount.isWorldMountReady(worldId);
 * ```
 */
export function isWorldMountReady(worldId: WorldId): boolean {
  return isWorldReady(asWorldNumber(worldId));
}

/**
 * Waits until a world reports mount readiness.
 *
 * Note: this helper only polls local readiness; callers still must keep
 * driving `tick(...)` while waiting.
 *
 * @returns `true` when ready, or `false` when timeout is reached.
 *
 * @example
 * ```ts
 * import { Mount, tick } from '@vulfram/engine';
 *
 * const ready = await Mount.waitWorldReady(worldId, {
 *   onPoll: () => tick(performance.now(), 16.67)
 * });
 * ```
 */
export async function waitWorldReady(
  worldId: WorldId,
  options: WaitWorldReadyOptions = {}
): Promise<boolean> {
  const timeoutMs = options.timeoutMs ?? 5_000;
  const pollIntervalMs = options.pollIntervalMs ?? 16;
  const deadline = Date.now() + timeoutMs;

  const poll = async (): Promise<boolean> => {
    if (isWorldMountReady(worldId)) return true;
    if (Date.now() >= deadline) return isWorldMountReady(worldId);

    if (options.onPoll) {
      await options.onPoll();
      if (isWorldMountReady(worldId)) return true;
    }

    await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
    return poll();
  };

  return poll();
}

function allocateTargetId(): TargetId {
  return asTargetId(engineState.nextGlobalId++);
}

function defaultLayout(): TargetLayerLayout {
  return {
    left: { unit: 'percent', value: 0 },
    top: { unit: 'percent', value: 0 },
    width: { unit: 'percent', value: 100 },
    height: { unit: 'percent', value: 100 },
    enabled: true,
    opacity: 1,
    zIndex: 0,
    blendMode: 0
  };
}

/**
 * Mounts a world into any target supported by core.
 *
 * This function hides realm details from the public API. Internally it:
 * 1) creates/updates target (`cmd-target-upsert`)
 * 2) enqueues layer bind (`cmd-target-layer-upsert`) in the same frame
 *    so command order remains deterministic and avoids duplicate binds.
 *
 * @param worldId World to be mounted.
 * @param args Target configuration and optional layout/camera/environment overrides.
 * @returns Generated target id and command ids for upsert + bind.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * const mount = Mount.mountWorld(worldId, {
 *   target: { kind: 'window', windowId }
 * });
 * ```
 */
export function mountWorld(
  worldId: WorldId,
  args: MountWorldArgs
): {
  targetId: TargetId;
  targetCommandId: CommandId;
  mountCommandId: CommandId;
} {
  const targetId = args.targetId ?? allocateTargetId();
  const world = asWorldNumber(worldId);

  const targetCommandId = upsertTarget(world, {
    targetId: targetId as number,
    ...args.target
  });

  const bindArgs = {
    targetId: targetId as number,
    layout: args.layout ?? defaultLayout(),
    cameraId: args.cameraId,
    environmentId: args.environmentId
  };
  const mountCommandId = bindWorldToTarget(world, bindArgs);

  return {
    targetId,
    targetCommandId: asCommandId(targetCommandId),
    mountCommandId: asCommandId(mountCommandId)
  };
}

/**
 * Convenience wrapper for mounting a world directly into a window target.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * Mount.mountWorldToWindow(worldId, windowId);
 * ```
 */
export function mountWorldToWindow(
  worldId: WorldId,
  windowId: WindowId,
  options: Omit<MountWorldArgs, 'target'> = {}
): {
  targetId: TargetId;
  targetCommandId: CommandId;
  mountCommandId: CommandId;
} {
  return mountWorld(worldId, {
    ...options,
    target: {
      kind: 'window',
      windowId: windowId as number
    }
  });
}

/**
 * Unmounts a world from a target.
 *
 * @param worldId Mounted world identifier.
 * @param targetId Target to unbind.
 * @returns Command id for unbind request.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * Mount.unmountWorld(worldId, targetId);
 * ```
 */
export function unmountWorld(worldId: WorldId, targetId: TargetId): CommandId {
  const commandId = unbindWorldFromTarget(asWorldNumber(worldId), {
    targetId: targetId as number
  } as Omit<CmdTargetLayerDisposeArgs, 'realmId'>);
  return asCommandId(commandId);
}

/**
 * Remounts an existing world->target binding with a new layer layout/camera/environment.
 *
 * @param worldId Mounted world identifier.
 * @param args Layer update payload.
 * @returns Command id for target-layer upsert.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * Mount.remountWorld(worldId, {
 *   targetId,
 *   layout: {
 *     left: { unit: 'percent', value: 0 },
 *     top: { unit: 'percent', value: 0 },
 *     width: { unit: 'percent', value: 100 },
 *     height: { unit: 'percent', value: 100 }
 *   }
 * });
 * ```
 */
export function remountWorld(
  worldId: WorldId,
  args: Omit<CmdTargetLayerUpsertArgs, 'realmId'>
): CommandId {
  const commandId = bindWorldToTarget(asWorldNumber(worldId), args);
  return asCommandId(commandId);
}

/**
 * Requests measurement for a mounted target.
 *
 * @example
 * ```ts
 * import { Mount } from '@vulfram/engine';
 *
 * Mount.measureMountedTarget(worldId, { targetId });
 * ```
 */
export function measureMountedTarget(worldId: WorldId, args: CmdTargetMeasurementArgs): CommandId {
  const commandId = measureTarget(asWorldNumber(worldId), args);
  return asCommandId(commandId);
}
