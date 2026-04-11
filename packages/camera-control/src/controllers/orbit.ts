import type { EntityId, World3DId } from '@vulfram/engine/world3d';
import { quat, vec3, type ReadonlyVec3 } from '@vulfram/engine/math';
import {
  applyLookAtIfEnabled,
  applyToWorld,
  cloneCameraTarget,
  createCameraTarget,
  runPipeline
} from '../core/pipeline';
import { readPointerState } from '../core/pointer';
import { clamp, makeLookRotation, sphericalToCartesian } from '../core/math';
import type {
  BaseCameraControllerHandle,
  CameraControllerOptions,
  CameraTarget,
  LookAtState
} from '../core/types';
import { weightOrDefault } from '../core/actions';

export interface OrbitControllerConfig extends CameraControllerOptions {
  target?: ReadonlyVec3;
  radius?: number;
  yaw?: number;
  pitch?: number;
  rotateSpeed?: number;
  pointerDeltaSensitivity?: number;
  invertPointerX?: boolean;
  invertPointerY?: boolean;
  panSpeed?: number;
  zoomSpeed?: number;
  zoomSensitivity?: number;
  minRadius?: number;
  maxRadius?: number;
  minPitch?: number;
  maxPitch?: number;
  enabled?: boolean;
}

export interface OrbitControllerHandle extends BaseCameraControllerHandle {
  toZoom(weight?: number): void;
  enable(): void;
  disable(): void;
  setEnabled(enabled: boolean): void;
  isEnabled(): boolean;
}

export function createOrbitController(
  worldId: World3DId,
  cameraEntityId: EntityId,
  config: OrbitControllerConfig = {}
): OrbitControllerHandle {
  const target = vec3.clone(config.target ?? [0, 0, 0]);
  let radius = config.radius ?? 5;
  let yaw = config.yaw ?? 0;
  let pitch = config.pitch ?? 0;
  let enabled = config.enabled ?? true;

  const rotateSpeed = config.rotateSpeed ?? 0.01;
  const pointerDeltaSensitivity = Math.max(0, config.pointerDeltaSensitivity ?? 1);
  const pointerXSign = config.invertPointerX ? -1 : 1;
  const pointerYSign = config.invertPointerY ? -1 : 1;
  const panSpeed = config.panSpeed ?? 0.005;
  const zoomSpeed = config.zoomSpeed ?? 0.03;
  const zoomSensitivity = Math.max(0, config.zoomSensitivity ?? 1);
  const minRadius = config.minRadius ?? 0.05;
  const maxRadius = config.maxRadius ?? 1_000;
  const minPitch = config.minPitch ?? -Math.PI * 0.49;
  const maxPitch = config.maxPitch ?? Math.PI * 0.49;

  const lookAtState: LookAtState = {
    enabled: false,
    target: vec3.create(),
    weight: 0
  };

  let zoomImpulse = 0;
  const offset = vec3.create();

  function composeRawTarget(): CameraTarget {
    sphericalToCartesian(offset, yaw, pitch, radius);
    const position = vec3.create();
    vec3.add(position, target, offset);
    const rotation = quat.create();
    makeLookRotation(rotation, position, target);
    return createCameraTarget(position, rotation);
  }

  let previousApplied = composeRawTarget();
  applyToWorld(worldId, cameraEntityId, previousApplied);

  return {
    update(dtSeconds: number): void {
      if (!enabled) {
        return;
      }
      const pointer = readPointerState(worldId);
      const pointerX = pointer.delta[0] * pointerXSign;
      const pointerY = pointer.delta[1] * pointerYSign;
      const lookX = pointer.lookDelta[0] * pointerXSign;
      const lookY = pointer.lookDelta[1] * pointerYSign;
      const rotateGesture = pointer.rightPressed || (pointer.leftPressed && !pointer.middlePressed);

      const panGesture = pointer.middlePressed && !pointer.rightPressed;

      if (rotateGesture) {
        const weightedLookX = lookX * pointerDeltaSensitivity;
        const weightedLookY = lookY * pointerDeltaSensitivity;
        yaw -= weightedLookX * rotateSpeed;
        pitch = clamp(pitch + weightedLookY * rotateSpeed, minPitch, maxPitch);
      }

      if (panGesture) {
        const right = vec3.create();
        const up = vec3.create();
        const currentRotation = quat.create();
        sphericalToCartesian(offset, yaw, pitch, radius);
        const eye = vec3.create();
        vec3.add(eye, target, offset);
        makeLookRotation(currentRotation, eye, target);

        vec3.transformQuat(right, [1, 0, 0], currentRotation);
        vec3.transformQuat(up, [0, 1, 0], currentRotation);

        const panRight = -pointerX * panSpeed * radius;
        const panUp = pointerY * panSpeed * radius;
        vec3.scaleAndAdd(target, target, right, panRight);
        vec3.scaleAndAdd(target, target, up, panUp);
      }

      let zoomWeight = zoomImpulse;
      if (pointer.leftPressed && pointer.rightPressed) {
        zoomWeight += pointerY;
      }
      zoomWeight *= zoomSensitivity;
      radius = clamp(radius * (1 + zoomWeight * zoomSpeed * dtSeconds * 60), minRadius, maxRadius);

      const raw = composeRawTarget();
      applyLookAtIfEnabled(raw, lookAtState, dtSeconds);

      const weightedLookX = lookX * pointerDeltaSensitivity;
      const weightedLookY = lookY * pointerDeltaSensitivity;
      const nextApplied = runPipeline(raw, previousApplied, config, {
        kind: 'orbit',
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
          lookAt: lookAtState.weight
        }
      });

      applyToWorld(worldId, cameraEntityId, nextApplied);
      previousApplied = cloneCameraTarget(nextApplied);
      zoomImpulse = 0;
    },

    lookAt(position: ReadonlyVec3, weight?: number): void {
      vec3.set(lookAtState.target, position[0] ?? 0, position[1] ?? 0, position[2] ?? 0);
      lookAtState.enabled = true;
      lookAtState.weight = weightOrDefault(weight);
    },

    toZoom(weight?: number): void {
      zoomImpulse += weightOrDefault(weight);
    },

    enable(): void {
      enabled = true;
    },

    disable(): void {
      enabled = false;
    },

    setEnabled(nextEnabled: boolean): void {
      enabled = nextEnabled;
    },

    isEnabled(): boolean {
      return enabled;
    }
  };
}
