import {
  get3DPointerDelta,
  is3DPointerButtonPressed,
  type World3DId
} from '@vulfram/engine/world3d';
import { vec3 } from '@vulfram/engine/math';
import type { CameraPointerState } from './types';

const POINTER_DELTA_DEADZONE = 0.01;

export function readPointerState(worldId: World3DId): CameraPointerState {
  const globalDelta = get3DPointerDelta(worldId);
  const deltaX = Math.abs(globalDelta[0]) <= POINTER_DELTA_DEADZONE ? 0 : globalDelta[0];
  const deltaY = Math.abs(globalDelta[1]) <= POINTER_DELTA_DEADZONE ? 0 : globalDelta[1];
  const delta = vec3.fromValues(deltaX, deltaY, 0);
  const lookDelta = vec3.clone(delta);
  const leftPressed = is3DPointerButtonPressed(worldId, 0);
  const middlePressed = is3DPointerButtonPressed(worldId, 1);
  const rightPressed = is3DPointerButtonPressed(worldId, 2);

  return {
    delta,
    lookDelta,
    leftPressed,
    middlePressed,
    rightPressed
  };
}
