import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import type { Intent } from '../../ecs';

/**
 * Emits an intent to the specified world.
 * Intents are processed by systems at the beginning of each tick.
 */
export function emitIntent(worldId: number, intent: Intent): void {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  world.intentStore.enqueue(intent);
}
