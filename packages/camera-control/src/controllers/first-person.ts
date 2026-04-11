import type { EntityId, World3DId } from '@vulfram/engine/world3d';
import { quat, vec3, type Quat, type ReadonlyVec3 } from '@vulfram/engine/math';
import {
  clearMotionImpulse,
  createMotionActionState,
  resolveMotionWeights,
  weightOrDefault
} from '../core/actions';
import { clamp, localBasisFromQuat } from '../core/math';
import {
  applyLookAtIfEnabled,
  applyToWorld,
  cloneCameraTarget,
  createCameraTarget,
  runPipeline
} from '../core/pipeline';
import { readPointerState } from '../core/pointer';
import type {
  CameraControllerOptions,
  LookAtState,
  MotionCameraControllerHandle
} from '../core/types';

export interface FirstPersonControllerConfig extends CameraControllerOptions {
  position?: ReadonlyVec3;
  yaw?: number;
  pitch?: number;
  moveSpeed?: number;
  lookSpeed?: number;
  pointerLookSpeed?: number;
  pointerDeltaSensitivity?: number;
  invertPointerX?: boolean;
  invertPointerY?: boolean;
  alwaysLook?: boolean;
  minPitch?: number;
  maxPitch?: number;
  allowVertical?: boolean;
}

export function createFirstPersonController(
  worldId: World3DId,
  cameraEntityId: EntityId,
  config: FirstPersonControllerConfig = {}
): MotionCameraControllerHandle {
  const position = vec3.clone(config.position ?? [0, 1.7, 4]);
  let yaw = config.yaw ?? 0;
  let pitch = config.pitch ?? 0;

  const moveSpeed = config.moveSpeed ?? 3.5;
  const lookSpeed = config.lookSpeed ?? 0.0025;
  const pointerLookSpeed = config.pointerLookSpeed ?? 1;
  const pointerDeltaSensitivity = Math.max(0, config.pointerDeltaSensitivity ?? 1);
  const pointerXSign = config.invertPointerX ? -1 : 1;
  const pointerYSign = config.invertPointerY ? -1 : 1;
  const alwaysLook = config.alwaysLook ?? false;
  const minPitch = config.minPitch ?? -Math.PI * 0.49;
  const maxPitch = config.maxPitch ?? Math.PI * 0.49;
  const allowVertical = config.allowVertical ?? false;

  const actions = createMotionActionState();
  const lookAtState: LookAtState = {
    enabled: false,
    target: vec3.create(),
    weight: 0
  };

  function composeRotation(): Quat {
    const out = quat.create();
    quat.rotateY(out, out, yaw);
    quat.rotateX(out, out, pitch);
    return out;
  }

  let previousApplied = createCameraTarget(position, composeRotation());
  applyToWorld(worldId, cameraEntityId, previousApplied);

  return {
    update(dtSeconds: number): void {
      const pointer = readPointerState(worldId);
      const pointerX = pointer.lookDelta[0] * pointerXSign;
      const pointerY = pointer.lookDelta[1] * pointerYSign;
      if (alwaysLook || pointer.rightPressed || pointer.leftPressed) {
        actions.impulse.lookX += pointerX * pointerLookSpeed * pointerDeltaSensitivity;
        actions.impulse.lookY += pointerY * pointerLookSpeed * pointerDeltaSensitivity;
      }

      const weights = resolveMotionWeights(actions, 0, lookAtState.weight);
      yaw -= weights.lookX * lookSpeed;
      pitch = clamp(pitch - weights.lookY * lookSpeed, minPitch, maxPitch);

      const rotation = composeRotation();
      const forward = vec3.create();
      const right = vec3.create();
      const up = vec3.create();
      localBasisFromQuat(rotation, forward, right, up);

      forward[1] = 0;
      right[1] = 0;
      if (vec3.length(forward) > 0) vec3.normalize(forward, forward);
      if (vec3.length(right) > 0) vec3.normalize(right, right);

      const moveVector = vec3.create();
      vec3.scaleAndAdd(moveVector, moveVector, forward, weights.forward);
      vec3.scaleAndAdd(moveVector, moveVector, right, weights.right);
      if (allowVertical) {
        vec3.scaleAndAdd(moveVector, moveVector, [0, 1, 0], weights.up);
      }

      if (vec3.length(moveVector) > 0) {
        vec3.normalize(moveVector, moveVector);
      }
      vec3.scaleAndAdd(position, position, moveVector, moveSpeed * dtSeconds);

      const raw = createCameraTarget(position, rotation);
      applyLookAtIfEnabled(raw, lookAtState, dtSeconds);

      const nextApplied = runPipeline(raw, previousApplied, config, {
        kind: 'first-person',
        worldId,
        cameraEntityId,
        dtSeconds,
        pointer,
        weights
      });

      applyToWorld(worldId, cameraEntityId, nextApplied);
      previousApplied = cloneCameraTarget(nextApplied);
      vec3.copy(position, nextApplied.position);

      clearMotionImpulse(actions);
    },

    lookAt(target: ReadonlyVec3, weight?: number): void {
      vec3.set(lookAtState.target, target[0], target[1], target[2]);
      lookAtState.enabled = true;
      lookAtState.weight = weightOrDefault(weight);
    },

    pressForward(weight?: number): void {
      actions.persistent.forward = weightOrDefault(weight);
    },
    releaseForward(): void {
      actions.persistent.forward = 0;
    },
    pressBackward(weight?: number): void {
      actions.persistent.backward = weightOrDefault(weight);
    },
    releaseBackward(): void {
      actions.persistent.backward = 0;
    },
    pressLeft(weight?: number): void {
      actions.persistent.left = weightOrDefault(weight);
    },
    releaseLeft(): void {
      actions.persistent.left = 0;
    },
    pressRight(weight?: number): void {
      actions.persistent.right = weightOrDefault(weight);
    },
    releaseRight(): void {
      actions.persistent.right = 0;
    },
    pressUp(weight?: number): void {
      actions.persistent.up = weightOrDefault(weight);
    },
    releaseUp(): void {
      actions.persistent.up = 0;
    },
    pressDown(weight?: number): void {
      actions.persistent.down = weightOrDefault(weight);
    },
    releaseDown(): void {
      actions.persistent.down = 0;
    },

    toForward(weight?: number): void {
      actions.impulse.forward += weightOrDefault(weight);
    },
    toBackward(weight?: number): void {
      actions.impulse.backward += weightOrDefault(weight);
    },
    toLeft(weight?: number): void {
      actions.impulse.left += weightOrDefault(weight);
    },
    toRight(weight?: number): void {
      actions.impulse.right += weightOrDefault(weight);
    },
    toUp(weight?: number): void {
      actions.impulse.up += weightOrDefault(weight);
    },
    toDown(weight?: number): void {
      actions.impulse.down += weightOrDefault(weight);
    },

    look(deltaX: number, deltaY: number, weight?: number): void {
      const w = weightOrDefault(weight);
      actions.impulse.lookX += deltaX * w;
      actions.impulse.lookY += deltaY * w;
    }
  };
}
