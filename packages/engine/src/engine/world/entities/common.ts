import { getWorldOrThrow } from '../../bridge/guards';
import { engineState } from '../../state';

export const WORLD_ENTITY_ID = 0;

export function allocateGlobalId(): number {
  return engineState.nextGlobalId++;
}

export function recalculateWorldWindowBindings(
  world: ReturnType<typeof getWorldOrThrow>,
): void {
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
