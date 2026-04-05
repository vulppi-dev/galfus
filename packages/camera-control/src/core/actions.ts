import type { CameraActionWeights } from './types';

type MotionPersistentState = {
  forward: number;
  backward: number;
  left: number;
  right: number;
  up: number;
  down: number;
};

type MotionImpulseState = {
  forward: number;
  backward: number;
  left: number;
  right: number;
  up: number;
  down: number;
  lookX: number;
  lookY: number;
};

export type MotionActionState = {
  persistent: MotionPersistentState;
  impulse: MotionImpulseState;
};

export function createMotionActionState(): MotionActionState {
  return {
    persistent: {
      forward: 0,
      backward: 0,
      left: 0,
      right: 0,
      up: 0,
      down: 0,
    },
    impulse: {
      forward: 0,
      backward: 0,
      left: 0,
      right: 0,
      up: 0,
      down: 0,
      lookX: 0,
      lookY: 0,
    },
  };
}

export function clearMotionImpulse(state: MotionActionState): void {
  state.impulse.forward = 0;
  state.impulse.backward = 0;
  state.impulse.left = 0;
  state.impulse.right = 0;
  state.impulse.up = 0;
  state.impulse.down = 0;
  state.impulse.lookX = 0;
  state.impulse.lookY = 0;
}

export function resolveMotionWeights(
  state: MotionActionState,
  zoom: number,
  lookAt: number,
): CameraActionWeights {
  const forward =
    state.persistent.forward -
    state.persistent.backward +
    state.impulse.forward -
    state.impulse.backward;

  const right =
    state.persistent.right -
    state.persistent.left +
    state.impulse.right -
    state.impulse.left;

  const up =
    state.persistent.up -
    state.persistent.down +
    state.impulse.up -
    state.impulse.down;

  return {
    forward,
    right,
    up,
    zoom,
    lookX: state.impulse.lookX,
    lookY: state.impulse.lookY,
    lookAt,
  };
}

export function weightOrDefault(weight?: number): number {
  return weight ?? 1.0;
}
