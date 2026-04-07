import type { InputStateComponent } from '../../ecs/components';
import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import type { RoutedPointerSnapshot } from './types';

const WORLD_ENTITY_ID = 0;

function getInputState(worldId: number): InputStateComponent | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('InputState') as InputStateComponent | undefined;
}

function fromInputState(state: InputStateComponent): RoutedPointerSnapshot {
  return {
    pointerTargetId: state.pointerTargetId,
    pointerTargetPosition: state.pointerPositionTarget,
    pointerTargetDelta: state.pointerTargetDelta,
    pointerTargetUv: state.pointerTargetUv,
    pointerTargetSize: state.pointerTargetSize
  };
}

export function getRoutedPointerSnapshotByWorld(worldId: number): RoutedPointerSnapshot | null {
  const state = getInputState(worldId);
  if (!state) return null;
  return fromInputState(state);
}

export function getRoutedPointerSnapshotByTarget(
  worldId: number,
  targetId: number
): RoutedPointerSnapshot | null {
  const state = getInputState(worldId);
  if (!state) return null;
  if (state.pointerTargetId !== targetId) return null;
  return fromInputState(state);
}
