import { EngineError } from '../errors';
import { engineState, type WorldState } from '../state';

/** Ensures engine runtime is initialized and not disposed. */
export function requireInitialized(): void {
  if (engineState.status === 'disposed') {
    throw new EngineError('Disposed', 'Engine has been disposed.');
  }
  if (engineState.status !== 'initialized') {
    throw new EngineError('NotInitialized', 'Engine is not initialized.');
  }
}

/** Resolves world state or throws when world id is unknown. */
export function getWorldOrThrow(worldId: number): WorldState {
  const world = engineState.worlds.get(worldId);
  if (!world) {
    throw new EngineError('WorldNotFound', `World ${worldId} not found.`);
  }
  return world;
}

/** Ensures entity exists in world entity set. */
export function ensureEntity(world: WorldState, entityId: number): void {
  if (!world.entities.has(entityId)) {
    throw new EngineError(
      'EntityNotFound',
      `Entity ${entityId} not found in World ${world.worldId}.`,
    );
  }
}
