import { update3DTransform, type EntityId, type World3DId } from '@vulfram/engine/world3d';
import { quat, vec3, type ReadonlyQuat, type ReadonlyVec3 } from 'gl-matrix';
import { makeLookRotation, slerpArc, smoothStepAlpha } from './math';
import type {
  CameraControllerContext,
  CameraControllerOptions,
  CameraTarget,
  LookAtState
} from './types';

export function createCameraTarget(position: ReadonlyVec3, rotation: ReadonlyQuat): CameraTarget {
  return {
    position: vec3.fromValues(position[0], position[1], position[2]),
    rotation: quat.fromValues(rotation[0], rotation[1], rotation[2], rotation[3])
  };
}

export function cloneCameraTarget(target: CameraTarget): CameraTarget {
  return {
    position: vec3.clone(target.position),
    rotation: quat.clone(target.rotation)
  };
}

export function applyLookAtIfEnabled(
  next: CameraTarget,
  lookAtState: LookAtState,
  dtSeconds: number
): void {
  if (!lookAtState.enabled || lookAtState.weight === 0) {
    return;
  }

  const desired = quat.create();
  makeLookRotation(desired, next.position, lookAtState.target);

  const alpha = smoothStepAlpha(lookAtState.weight, dtSeconds);
  const longArc = lookAtState.weight < 0;
  slerpArc(next.rotation, next.rotation, desired, alpha, longArc);
}

export function runPipeline(
  nextRaw: CameraTarget,
  prevApplied: CameraTarget,
  options: CameraControllerOptions,
  context: CameraControllerContext
): CameraTarget {
  const strategized = options.translationStrategy
    ? options.translationStrategy(nextRaw, prevApplied, context)
    : nextRaw;

  return options.easing ? options.easing(strategized, prevApplied, context) : strategized;
}

export function applyToWorld(
  worldId: World3DId,
  cameraEntityId: EntityId,
  next: CameraTarget
): void {
  update3DTransform(worldId, cameraEntityId, {
    position: next.position,
    rotation: next.rotation
  });
}
