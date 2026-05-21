import type { EngineTransportFactory } from '@galfus/transport-types';
import {
  collectCommands,
  enqueueGlobalCommand,
  markRoutingIndexDirty,
  routeEvents,
  routeResponses
} from './bridge/dispatch';
import { requireInitialized } from './bridge/guards';
import { deserializeEvents, deserializeResponses, serializeBatch } from './bridge/protocol';
import type { ComponentSchema, System, SystemContext, SystemStep } from './ecs';
import { EngineError } from './errors';
import { engineState, REQUIRED_SYSTEMS, type WorldState } from './state';
import { createIntentStore } from './intents/store';
import * as CoreSystems from './systems';
import type { UploadType } from '../types/kinds';
import type { RealmKind } from '../types/cmds/realm';
import type {
  CmdSystemBuildVersionGetArgs,
  CmdSystemDiagnosticsSetArgs,
  CmdSystemLogLevelGetArgs,
  CmdSystemLogLevelSetArgs,
  CmdUploadBufferDiscardAllArgs
} from '../types/cmds/system';
import { asWorldId, type WorldId } from './world/types';

export * from './window/manager';

function recalculateWorldWindowBindings(world: WorldState): void {
  world.boundWindowIds.clear();
  for (const windowId of world.targetWindowBindings.values()) {
    world.boundWindowIds.add(windowId);
  }

  if (world.boundWindowIds.size === 0) {
    world.primaryWindowId = undefined;
    return;
  }

  let primary = Number.POSITIVE_INFINITY;
  for (const windowId of world.boundWindowIds) {
    if (windowId < primary) {
      primary = windowId;
    }
  }
  world.primaryWindowId = primary;
}

/**
 * Shared realm creation options used by `createWorld3D`.
 */
export type CreateWorldOptions = {
  /** Core scheduling hint for realm priority. */
  importance?: number;
  /** Core-side cache policy value. */
  cachePolicy?: number;
  /** Bit flags forwarded to realm creation in core. */
  flags?: number;
};

/**
 * Initializes the engine runtime and registers core systems.
 * Call once before creating worlds or issuing commands.
 *
 * @example
 * ```ts
 * import { initEngine } from '@galfus/engine/core';
 * import { createBrowserTransport } from '@galfus/transport-browser';
 *
 * initEngine({
 *   transport: () => createBrowserTransport()
 * });
 * ```
 */
export function initEngine(config: {
  /** Transport factory for the runtime (WASM, Bun, N-API, etc.). */
  transport: EngineTransportFactory;
  /** Enables verbose debug logs and extra diagnostics. */
  debug?: boolean;
}): void {
  if (engineState.status === 'initialized') {
    throw new EngineError('AlreadyInitialized', 'Engine already initialized.');
  }

  // Reset Engine State for fresh start (important if re-initializing after dispose)
  engineState.worlds.clear();
  engineState.commandBatch = [];
  engineState.usedWindowIds.clear();
  engineState.confirmedWindowIds.clear();
  engineState.pendingWindowCreateByCommandId.clear();
  engineState.pendingWindowCloseByCommandId.clear();
  engineState.globalPendingCommands = [];
  engineState.globalPendingCommandsHead = 0;
  engineState.commandTracker.clear();
  engineState.globalCommandTracker.clear();
  engineState.globalInboundResponses = [];
  engineState.nextWorldId = 1;
  engineState.nextWindowId = 1;
  engineState.nextEntityId = 1;
  engineState.nextCommandId = 1;
  engineState.nextGlobalId = 100;
  engineState.routingIndex.byWindowId.clear();
  engineState.routingIndex.byRealmId.clear();
  engineState.routingIndex.byTargetId.clear();
  engineState.routingIndex.dirty = true;
  engineState.registry.systems.input = [];
  engineState.registry.systems.update = [];
  engineState.registry.systems.preRender = [];
  engineState.registry.systems.postRender = [];

  engineState.flags.debugEnabled = config.debug ?? false;

  const transport = config.transport();
  const result = transport.galfusInit();
  if (result !== 0) {
    throw new EngineError('InitFailed', `galfusInit failed with code ${result}.`);
  }

  engineState.transport = transport;
  engineState.status = 'initialized';

  // Register Core Systems
  registerSystem('input', CoreSystems.InputMirrorSystem);
  registerSystem('update', CoreSystems.CommandIntentSystem);
  registerSystem('update', CoreSystems.WorldLifecycleSystem);
  registerSystem('update', CoreSystems.ResourceUploadSystem);
  registerSystem('preRender', CoreSystems.ConstraintSolveSystem);
  registerSystem('preRender', CoreSystems.SceneSyncSystem);
  registerSystem('postRender', CoreSystems.ResponseDecodeSystem);
  registerSystem('postRender', CoreSystems.DiagnosticsSystem);
}

/**
 * Disposes the engine and releases the active transport.
 * After dispose, you must call initEngine again to use the engine.
 *
 * @example
 * ```ts
 * import { disposeEngine } from '@galfus/engine/core';
 *
 * disposeEngine();
 * ```
 */
export function disposeEngine(): void {
  requireInitialized();
  const transport = engineState.transport;
  if (transport) {
    transport.galfusDispose();
  }
  engineState.transport = null;
  engineState.worlds.clear();
  engineState.commandBatch = [];
  engineState.usedWindowIds.clear();
  engineState.confirmedWindowIds.clear();
  engineState.pendingWindowCreateByCommandId.clear();
  engineState.pendingWindowCloseByCommandId.clear();
  engineState.globalPendingCommands = [];
  engineState.globalPendingCommandsHead = 0;
  engineState.commandTracker.clear();
  engineState.globalCommandTracker.clear();
  engineState.globalInboundResponses = [];
  engineState.routingIndex.byWindowId.clear();
  engineState.routingIndex.byRealmId.clear();
  engineState.routingIndex.byTargetId.clear();
  engineState.routingIndex.dirty = true;
  engineState.status = 'disposed';
}

/**
 * Registers a custom component schema for editor tooling or extensions.
 *
 * @example
 * ```ts
 * import { registerComponent } from '@galfus/engine/core';
 *
 * registerComponent('Health', {
 *   fields: {
 *     current: 'number',
 *     max: 'number'
 *   }
 * });
 * ```
 */
export function registerComponent(name: string, schema: ComponentSchema): void {
  requireInitialized();
  engineState.registry.components.set(name, schema);
}

/**
 * Registers a custom system into the engine pipeline.
 *
 * Stage semantics:
 * - `input`: consume mirrored inbound events/state for the current frame.
 * - `update`: mutate ECS state and emit intents.
 * - `preRender`: resolve constraints and emit core commands.
 * - `postRender`: consume command responses and diagnostics.
 *
 * @example
 * ```ts
 * import { registerSystem } from '@galfus/engine/core';
 *
 * registerSystem('update', (world, context) => {
 *   void world;
 *   void context;
 * });
 * ```
 */
export function registerSystem(step: SystemStep, system: System): void {
  requireInitialized();
  engineState.registry.systems[step].push(system);
}

function uploadTypeToId(type: UploadType): number {
  switch (type) {
    case 'raw':
      return 0;
    case 'shader-source':
      return 1;
    case 'geometry-data':
      return 2;
    case 'vertex-data':
      return 3;
    case 'index-data':
      return 4;
    case 'image-data':
      return 5;
    case 'binary-asset':
      return 6;
  }
}

/**
 * Uploads a raw buffer to the core for later use (textures, geometry, etc.).
 *
 * @example
 * ```ts
 * import { uploadBuffer } from '@galfus/engine/core';
 *
 * uploadBuffer(10, 'image-data', pngBytes);
 * ```
 */
export function uploadBuffer(bufferId: number, type: UploadType, data: Uint8Array): void {
  requireInitialized();
  const transport = engineState.transport!;
  const result = transport.galfusUploadBuffer(bufferId, uploadTypeToId(type), data);
  if (result !== 0) {
    throw new EngineError(
      'UploadFailed',
      `galfusUploadBuffer failed for ID ${bufferId} with code ${result}.`
    );
  }
}

/**
 * Configures global runtime diagnostics and pointer tracing.
 *
 * @example
 * ```ts
 * import { setSystemDiagnostics } from '@galfus/engine/core';
 *
 * setSystemDiagnostics({ enabled: true });
 * ```
 */
export function setSystemDiagnostics(args: CmdSystemDiagnosticsSetArgs): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-system-diagnostics-set', args);
}

/** Updates the global core log filter level. */
export function setSystemLogLevel(args: CmdSystemLogLevelSetArgs): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-system-log-level-set', args);
}

/** Returns the current global core log filter level. */
export function getSystemLogLevel(args: CmdSystemLogLevelGetArgs = {}): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-system-log-level-get', args);
}

/**
 * Requests the core to return build/runtime version information.
 *
 * @example
 * ```ts
 * import { getCoreBuildVersion } from '@galfus/engine/core';
 *
 * const commandId = getCoreBuildVersion();
 * ```
 */
export function getCoreBuildVersion(args: CmdSystemBuildVersionGetArgs = {}): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-system-build-version-get', args);
}

/**
 * Requests the core to discard all pending upload buffers.
 *
 * @example
 * ```ts
 * import { discardAllUploadBuffers } from '@galfus/engine/core';
 *
 * discardAllUploadBuffers();
 * ```
 */
export function discardAllUploadBuffers(args: CmdUploadBufferDiscardAllArgs = {}): number {
  requireInitialized();
  return enqueueGlobalCommand('cmd-upload-buffer-discard-all', args);
}

function createRealmWorld(kind: RealmKind, config: CreateWorldOptions = {}): WorldId {
  requireInitialized();

  const worldId = engineState.nextWorldId++;

  const world: WorldState = {
    worldId,
    realmKind: kind,
    primaryWindowId: undefined,
    boundWindowIds: new Set(),
    targetLayerBindings: new Map(),
    targetWindowBindings: new Map(),
    coreRealmId: undefined,
    realmCreateArgs: {
      kind,
      importance: config.importance,
      cachePolicy: config.cachePolicy,
      flags: config.flags
    },
    resolvedEntityTransforms: new Map(),
    constraintDirtyEntities: new Set(),
    constraintScratchResolved: new Map(),
    constraintScratchVisiting: new Set(),
    constraintChangedEntities: new Set(),
    constraintChildrenByParent: new Map(),
    constraintParentByChild: new Map(),
    sceneSyncMatrixScratch: new Map(),
    entities: new Set(),
    components: new Map(),
    nextCoreId: 100,
    systems: [...REQUIRED_SYSTEMS],
    intentStore: createIntentStore(),
    internalEvents: [],
    pendingCommands: [],
    pendingCommandsHead: 0,
    inboundEvents: [],
    inboundResponses: [],
    realmCreateRetryCount: 0,
    nextRealmCreateRetryAtMs: 0
  };

  engineState.worlds.set(worldId, world);
  markRoutingIndexDirty();
  world.pendingCommands.push({
    id: engineState.nextCommandId++,
    type: 'cmd-realm-create',
    content: world.realmCreateArgs
  });
  return asWorldId(worldId);
}

/**
 * Creates a `three-d` realm world and queues `cmd-realm-create`.
 *
 * This function only allocates local runtime state and enqueues the create command.
 * The core realm is resolved asynchronously after at least one `tick`.
 *
 * Preconditions:
 * - `initEngine` must have been called.
 *
 * Side effects:
 * - Allocates a new world ID.
 * - Registers the world in engine state.
 * - Enqueues `cmd-realm-create` for the new world.
 *
 * @param config Optional create options.
 * @returns Numeric world ID associated with a core `three-d` realm.
 */
export function createWorld3D(config?: CreateWorldOptions): WorldId {
  return createRealmWorld('three-d', config);
}

/**
 * Creates a default `three-d` world.
 *
 * @example
 * ```ts
 * import { createWorld } from '@galfus/engine/core';
 *
 * const worldId = createWorld();
 * ```
 */
export function createWorld(config?: CreateWorldOptions): WorldId {
  return createWorld3D(config);
}

/**
 * Advances the engine by one frame.
 * Call once per frame with monotonic time and delta in milliseconds.
 *
 * @example
 * ```ts
 * import { tick } from '@galfus/engine/core';
 *
 * tick(performance.now(), 16.67);
 * ```
 */
export function tick(timeMs: number, deltaMs: number): void {
  requireInitialized();
  const transport = engineState.transport!;
  const coreTimeMs = Math.max(0, Math.floor(timeMs));
  const coreDeltaMs = Math.max(0, Math.floor(deltaMs));

  // 1. Engine Phase: Receive from Core (Input Pipeline)
  // We process events and responses received since last frame
  const eventsResult = transport.galfusReceiveEvents();
  if (eventsResult.result === 0 && eventsResult.buffer.length > 0) {
    const events = deserializeEvents(eventsResult.buffer);
    routeEvents(events);
  }

  const responsesResult = transport.galfusReceiveQueue();
  if (responsesResult.result === 0 && responsesResult.buffer.length > 0) {
    const responses = deserializeResponses(responsesResult.buffer);
    routeResponses(responses);
  }
  processGlobalResponses();

  // 2. Engine Phase: Clock & Context Preparation
  const cappedDelta = Math.min(coreDeltaMs, 100);
  engineState.clock.lastTime = timeMs;
  engineState.clock.lastDelta = cappedDelta;
  engineState.clock.frameCount++;

  const context: SystemContext = {
    dt: cappedDelta / 1000,
    time: timeMs / 1000,
    worldId: 0
  };

  // 3. World Phase: System Execution
  engineState.flags.isExecutingSystems = true;
  try {
    for (const [worldId, world] of engineState.worlds) {
      context.worldId = worldId;
      executeSystemStep(world, context, 'input');
      executeSystemStep(world, context, 'update');
      executeSystemStep(world, context, 'preRender');
      executeSystemStep(world, context, 'postRender');
    }
  } finally {
    engineState.flags.isExecutingSystems = false;
  }

  // 4. Engine Phase: Collect & Send (Output Pipeline)
  collectCommands();

  if (engineState.commandBatch.length > 0) {
    const batchBuffer = serializeBatch(engineState.commandBatch);
    const result = transport.galfusSendQueue(batchBuffer);
    if (result !== 0) {
      console.error(
        `[Galfus] galfusSendQueue failed with result ${result}. This usually indicates a MessagePack serialization mismatch between Host and Core.`
      );
      for (const cmd of engineState.commandBatch) {
        engineState.commandTracker.delete(cmd.id);
        engineState.globalCommandTracker.delete(cmd.id);
        const pendingCreateWindowId = engineState.pendingWindowCreateByCommandId.get(cmd.id);
        if (pendingCreateWindowId !== undefined) {
          engineState.usedWindowIds.delete(pendingCreateWindowId);
          engineState.pendingWindowCreateByCommandId.delete(cmd.id);
        }
        engineState.pendingWindowCloseByCommandId.delete(cmd.id);
      }
      if (engineState.flags.debugEnabled) {
        console.group('[Galfus Debug] Failed Batch');
        console.debug('Result:', result);
        console.debug('Batch Size:', batchBuffer.length, 'bytes');
        console.debug(
          'Command Types:',
          engineState.commandBatch.map((c) => c.type)
        );
        console.groupEnd();
      }
    }
    engineState.commandBatch = [];
  }

  // 5. Core Phase: Execute Tick
  transport.galfusTick(coreTimeMs, coreDeltaMs);
}

function processGlobalResponses(): void {
  for (let i = 0; i < engineState.globalInboundResponses.length; i++) {
    const res = engineState.globalInboundResponses[i]!;
    const content = res.content as { success?: boolean; message?: string };
    if (res.type === 'window-create') {
      const pendingWindowId = engineState.pendingWindowCreateByCommandId.get(res.id);
      if (pendingWindowId !== undefined) {
        if (content.success) {
          engineState.confirmedWindowIds.add(pendingWindowId);
        } else {
          engineState.usedWindowIds.delete(pendingWindowId);
        }
        engineState.pendingWindowCreateByCommandId.delete(res.id);
      }
    } else if (res.type === 'window-close') {
      const pendingWindowId = engineState.pendingWindowCloseByCommandId.get(res.id);
      if (pendingWindowId !== undefined) {
        if (content.success) {
          engineState.usedWindowIds.delete(pendingWindowId);
          engineState.confirmedWindowIds.delete(pendingWindowId);
          for (const world of engineState.worlds.values()) {
            for (const [targetId, windowId] of world.targetWindowBindings) {
              if (windowId === pendingWindowId) {
                world.targetWindowBindings.delete(targetId);
                world.targetLayerBindings.delete(targetId);
              }
            }
            recalculateWorldWindowBindings(world);
          }
          markRoutingIndexDirty();
        }
        engineState.pendingWindowCloseByCommandId.delete(res.id);
      }
    }
    if (content && typeof content.success === 'boolean' && !content.success) {
      console.error(`[Global] Command ${res.type} (ID: ${res.id}) failed: ${content.message}`);
    }
  }
  engineState.globalInboundResponses.length = 0;
}

/**
 * Executes a specific group of systems with error handling.
 */
function executeSystemStep(world: WorldState, context: SystemContext, step: SystemStep): void {
  const systems = engineState.registry.systems[step];
  for (const system of systems) {
    try {
      system(world, context);
    } catch (err) {
      console.error(
        `[SystemError] World ${context.worldId} | Step: ${step} | System: ${
          system.name || 'anonymous'
        }\n`,
        err
      );
    }
  }
}
