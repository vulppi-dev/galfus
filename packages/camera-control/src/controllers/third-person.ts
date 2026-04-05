import type { EntityId, World3DId } from '@vulfram/engine/world3d';
import { quat, vec3, type ReadonlyVec3 } from 'gl-matrix';
import {
  applyLookAtIfEnabled,
  applyToWorld,
  cloneCameraTarget,
  createCameraTarget,
  runPipeline,
} from '../core/pipeline';
import { readPointerState } from '../core/pointer';
import { clamp, makeLookRotation, sphericalToCartesian } from '../core/math';
import type {
  BaseCameraControllerHandle,
  CameraControllerOptions,
  CameraTarget,
  LookAtState,
} from '../core/types';
import { weightOrDefault } from '../core/actions';

export interface ThirdPersonControllerConfig extends CameraControllerOptions {
  target?: ReadonlyVec3;
  distance?: number;
  yaw?: number;
  pitch?: number;
  rotateSpeed?: number;
  pointerDeltaSensitivity?: number;
  invertPointerX?: boolean;
  invertPointerY?: boolean;
  zoomSpeed?: number;
  zoomSensitivity?: number;
  followSmoothing?: number;
  minDistance?: number;
  maxDistance?: number;
  alwaysLook?: boolean;
  minPitch?: number;
  maxPitch?: number;
}

export interface ThirdPersonControllerHandle extends BaseCameraControllerHandle {
  setTarget(position: ReadonlyVec3): void;
  toZoom(weight?: number): void;
}

export function createThirdPersonController(
  worldId: World3DId,
  cameraEntityId: EntityId,
  config: ThirdPersonControllerConfig = {},
): ThirdPersonControllerHandle {
  const target = vec3.clone(config.target ?? [0, 0, 0]);
  let distance = config.distance ?? 4;
  let yaw = config.yaw ?? 0;
  let pitch = config.pitch ?? 0.2;

  const rotateSpeed = config.rotateSpeed ?? 0.01;
  const pointerDeltaSensitivity = Math.max(0, config.pointerDeltaSensitivity ?? 1);
  const pointerXSign = config.invertPointerX ? -1 : 1;
  const pointerYSign = config.invertPointerY ? -1 : 1;
  const zoomSpeed = config.zoomSpeed ?? 0.03;
  const zoomSensitivity = Math.max(0, config.zoomSensitivity ?? 1);
  const followSmoothing = config.followSmoothing ?? 10;
  const alwaysLook = config.alwaysLook ?? false;
  const minDistance = config.minDistance ?? 0.5;
  const maxDistance = config.maxDistance ?? 100;
  const minPitch = config.minPitch ?? -Math.PI * 0.3;
  const maxPitch = config.maxPitch ?? Math.PI * 0.45;

  const lookAtState: LookAtState = {
    enabled: false,
    target: vec3.create(),
    weight: 0,
  };
  let zoomImpulse = 0;

  const smoothedTarget = vec3.clone(target);
  const offset = vec3.create();

  function composeRawTarget(): CameraTarget {
    sphericalToCartesian(offset, yaw, pitch, distance);
    const position = vec3.create();
    vec3.add(position, smoothedTarget, offset);
    const rotation = quat.create();
    makeLookRotation(rotation, position, smoothedTarget);
    return createCameraTarget(position, rotation);
  }

  let previousApplied = composeRawTarget();
  applyToWorld(worldId, cameraEntityId, previousApplied);

  return {
    update(dtSeconds: number): void {
      const pointer = readPointerState(worldId);
      const pointerX = pointer.delta[0] * pointerXSign;
      const pointerY = pointer.delta[1] * pointerYSign;
      const rotateGesture = alwaysLook || pointer.rightPressed || pointer.leftPressed;
      if (rotateGesture) {
        const lookX = pointerX * pointerDeltaSensitivity;
        const lookY = pointerY * pointerDeltaSensitivity;
        yaw -= lookX * rotateSpeed;
        pitch = clamp(pitch - lookY * rotateSpeed, minPitch, maxPitch);
      }

      let zoomWeight = zoomImpulse;
      if (pointer.leftPressed && pointer.rightPressed) {
        zoomWeight += pointerY;
      }
      zoomWeight *= zoomSensitivity;
      distance = clamp(
        distance * (1 + zoomWeight * zoomSpeed * dtSeconds * 60),
        minDistance,
        maxDistance,
      );

      const alpha = 1 - Math.exp(-followSmoothing * dtSeconds);
      vec3.lerp(smoothedTarget, smoothedTarget, target, alpha);

      const raw = composeRawTarget();
      applyLookAtIfEnabled(raw, lookAtState, dtSeconds);

      const weightedLookX = pointerX * pointerDeltaSensitivity;
      const weightedLookY = pointerY * pointerDeltaSensitivity;
      const nextApplied = runPipeline(raw, previousApplied, config, {
        kind: 'third-person',
        worldId,
        cameraEntityId,
        dtSeconds,
        pointer,
        weights: {
          forward: 0,
          right: 0,
          up: 0,
          zoom: zoomWeight,
          lookX: weightedLookX,
          lookY: weightedLookY,
          lookAt: lookAtState.weight,
        },
      });

      applyToWorld(worldId, cameraEntityId, nextApplied);
      previousApplied = cloneCameraTarget(nextApplied);
      zoomImpulse = 0;
    },

    lookAt(position: ReadonlyVec3, weight?: number): void {
      vec3.set(lookAtState.target, position[0], position[1], position[2]);
      lookAtState.enabled = true;
      lookAtState.weight = weightOrDefault(weight);
    },

    setTarget(position: ReadonlyVec3): void {
      vec3.set(target, position[0], position[1], position[2]);
    },

    toZoom(weight?: number): void {
      zoomImpulse += weightOrDefault(weight);
    },
  };
}
