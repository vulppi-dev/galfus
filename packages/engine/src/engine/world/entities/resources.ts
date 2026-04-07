import type { ShadowConfig } from '../../../types/cmds/shadow';
import type { EnvironmentConfig, CmdEnvironmentDisposeArgs } from '../../../types/cmds/environment';
import type {
  CmdRealmRenderGraphBindArgs,
  CmdRenderGraphDisposeArgs,
  CmdRenderGraphListArgs,
  CmdRenderGraphUpsertArgs
} from '../../../types/cmds/render-graph';
import type { NotificationLevel } from '../../../types/kinds';
import { EngineError } from '../../errors';
import { enqueueCommand } from '../../bridge/dispatch';
import { getWorldOrThrow } from '../../bridge/guards';
import { emitIntent } from './intents';

/**
 * Requests a list of resources from the engine for debugging.
 */
export function requestResourceList(
  worldId: number,
  resourceType: 'model' | 'material' | 'texture' | 'geometry' | 'light' | 'camera'
): void {
  emitIntent(worldId, {
    type: 'request-resource-list',
    resourceType
  });
}

function resolveWorldWindowId(worldId: number): number {
  const world = getWorldOrThrow(worldId);
  if (world.primaryWindowId === undefined) {
    for (const windowId of world.targetWindowBindings.values()) {
      return windowId;
    }
    throw new EngineError(
      'WindowNotFound',
      `World ${worldId} has no window binding available for this command.`
    );
  }
  return world.primaryWindowId;
}

/** Requests model list for a world window context. */
export function listModels(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-model-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Requests material list for a world window context. */
export function listMaterials(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-material-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Requests texture list for a world window context. */
export function listTextures(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-texture-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Requests geometry list for a world window context. */
export function listGeometries(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-geometry-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Requests light list for a world window context. */
export function listLights(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-light-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Requests camera list for a world window context. */
export function listCameras(worldId: number): number {
  return enqueueCommand(worldId, 'cmd-camera-list', {
    windowId: resolveWorldWindowId(worldId)
  });
}

/** Creates or updates a render graph in the core catalog. */
export function upsertRenderGraph(worldId: number, args: CmdRenderGraphUpsertArgs): number {
  return enqueueCommand(worldId, 'cmd-render-graph-upsert', args);
}

/** Disposes a render graph from the core catalog. */
export function disposeRenderGraph(worldId: number, args: CmdRenderGraphDisposeArgs): number {
  return enqueueCommand(worldId, 'cmd-render-graph-dispose', args);
}

/** Requests the current render graph catalog. */
export function listRenderGraphs(worldId: number, args: CmdRenderGraphListArgs = {}): number {
  return enqueueCommand(worldId, 'cmd-render-graph-list', args);
}

/** Binds a realm to a render graph id. */
export function bindRealmRenderGraph(worldId: number, args: CmdRealmRenderGraphBindArgs): number {
  return enqueueCommand(worldId, 'cmd-realm-render-graph-bind', args);
}

/** Disposes a material. */
export function disposeMaterial(worldId: number, resourceId: number): void {
  emitIntent(worldId, { type: 'dispose-material', resourceId });
}

/** Disposes a texture. */
export function disposeTexture(worldId: number, resourceId: number): void {
  emitIntent(worldId, { type: 'dispose-texture', resourceId });
}

/** Disposes a geometry. */
export function disposeGeometry(worldId: number, resourceId: number): void {
  emitIntent(worldId, { type: 'dispose-geometry', resourceId });
}

/** Sends a system notification. */
export function sendNotification(
  worldId: number,
  props: {
    level: NotificationLevel;
    title: string;
    message: string;
  }
): void {
  emitIntent(worldId, {
    type: 'send-notification',
    level: props.level,
    title: props.title,
    message: props.message
  });
}

/** Configures shadow settings for the world. */
export function configureShadows(worldId: number, config: ShadowConfig): void {
  emitIntent(worldId, {
    type: 'configure-shadows',
    config
  });
}

/** Configures environment settings for the world. */
export function configureEnvironment(worldId: number, config: EnvironmentConfig): void {
  emitIntent(worldId, {
    type: 'configure-environment',
    config
  });
}

/**
 * Disposes an environment profile.
 * If omitted, defaults to this world id (engine convention).
 */
export function disposeEnvironment(
  worldId: number,
  args: CmdEnvironmentDisposeArgs = { environmentId: worldId }
): number {
  return enqueueCommand(worldId, 'cmd-environment-dispose', args);
}
