import {
  get3DPointerDelta,
  is3DPointerButtonPressed,
  type World3DId
} from '@vulfram/engine/world3d';
import { vec3 } from 'gl-matrix';
import type { CameraPointerState } from './types';

const POINTER_DELTA_DEADZONE = 0.01;

export function readPointerState(worldId: World3DId): CameraPointerState {
  const globalDelta = get3DPointerDelta(worldId);
  // Look controllers should track window-relative pointer motion.
  // Target-relative deltas are useful for picking UX, but can distort camera look.
  const sourceDelta = globalDelta;
  const deltaX = Math.abs(sourceDelta[0]) <= POINTER_DELTA_DEADZONE ? 0 : sourceDelta[0];
  const deltaY = Math.abs(sourceDelta[1]) <= POINTER_DELTA_DEADZONE ? 0 : sourceDelta[1];

  const delta = vec3.fromValues(deltaX, deltaY, 0);
  const leftPressed = is3DPointerButtonPressed(worldId, 0);
  const middlePressed = is3DPointerButtonPressed(worldId, 1);
  const rightPressed = is3DPointerButtonPressed(worldId, 2);

  return {
    delta,
    leftPressed,
    middlePressed,
    rightPressed
  };
}
