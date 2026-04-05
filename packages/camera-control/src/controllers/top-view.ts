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
import { clamp, makeLookRotation } from '../core/math';
import type {
  BaseCameraControllerHandle,
  CameraControllerOptions,
  CameraTarget,
  LookAtState,
} from '../core/types';
import { weightOrDefault } from '../core/actions';

export interface TopViewControllerConfig extends CameraControllerOptions {
  focus?: ReadonlyVec3;
  height?: number;
  yaw?: number;
  pitch?: number;
  panSpeed?: number;
  rotateSpeed?: number;
  pointerDeltaSensitivity?: number;
  invertPointerX?: boolean;
  invertPointerY?: boolean;
  zoomSpeed?: number;
  zoomSensitivity?: number;
  minHeight?: number;
  maxHeight?: number;
  focusLocked?: boolean;
}

export interface TopViewControllerHandle extends BaseCameraControllerHandle {
  setFocus(position: ReadonlyVec3): void;
  toZoom(weight?: number): void;
  setFocusLocked(locked: boolean): void;
  isFocusLocked(): boolean;
}

export function createTopViewController(
  worldId: World3DId,
  cameraEntityId: EntityId,
  config: TopViewControllerConfig = {},
): TopViewControllerHandle {
  const focus = vec3.clone(config.focus ?? [0, 0, 0]);
  let height = config.height ?? 10;
  let yaw = config.yaw ?? 0;
  let pitch = config.pitch ?? -1.2;

  const panSpeed = config.panSpeed ?? 0.02;
  const rotateSpeed = config.rotateSpeed ?? 0.01;
  const pointerDeltaSensitivity = Math.max(0, config.pointerDeltaSensitivity ?? 1);
  const pointerXSign = config.invertPointerX ? -1 : 1;
  const pointerYSign = config.invertPointerY ? -1 : 1;
  const zoomSpeed = config.zoomSpeed ?? 0.02;
  const zoomSensitivity = Math.max(0, config.zoomSensitivity ?? 1);
  const minHeight = config.minHeight ?? 1;
  const maxHeight = config.maxHeight ?? 500;
  let focusLocked = config.focusLocked ?? false;

  const lookAtState: LookAtState = {
    enabled: false,
    target: vec3.create(),
    weight: 0,
  };
  let zoomImpulse = 0;

  function composeRawTarget(): CameraTarget {
    const local = vec3.fromValues(0, 0, height);
    const yRot = quat.create();
    quat.rotateY(yRot, yRot, yaw);
    const xRot = quat.create();
    quat.rotateX(xRot, xRot, pitch);
    const rotation = quat.create();
    quat.multiply(rotation, yRot, xRot);

    const position = vec3.clone(local);
    vec3.transformQuat(position, position, rotation);
    vec3.add(position, position, focus);

    const look = quat.create();
    makeLookRotation(look, position, focus);
    return createCameraTarget(position, look);
  }

  let previousApplied = composeRawTarget();
  applyToWorld(worldId, cameraEntityId, previousApplied);

  return {
    update(dtSeconds: number): void {
      const pointer = readPointerState(worldId);
      const pointerX = pointer.delta[0] * pointerXSign;
      const pointerY = pointer.delta[1] * pointerYSign;

      if (!focusLocked && pointer.leftPressed) {
        const panX = -pointerX * panSpeed * Math.max(1, height * 0.25);
        const panZ = pointerY * panSpeed * Math.max(1, height * 0.25);

        const right = vec3.fromValues(Math.cos(yaw), 0, -Math.sin(yaw));
        const forward = vec3.fromValues(Math.sin(yaw), 0, Math.cos(yaw));
        vec3.scaleAndAdd(focus, focus, right, panX);
        vec3.scaleAndAdd(focus, focus, forward, panZ);
      }

      if (pointer.rightPressed) {
        const lookX = pointerX * pointerDeltaSensitivity;
        yaw -= lookX * rotateSpeed;
      }

      let zoomWeight = zoomImpulse;
      if (pointer.leftPressed && pointer.rightPressed) {
        zoomWeight += pointerY;
      }
      zoomWeight *= zoomSensitivity;
      height = clamp(
        height * (1 + zoomWeight * zoomSpeed * dtSeconds * 60),
        minHeight,
        maxHeight,
      );

      const raw = composeRawTarget();
      applyLookAtIfEnabled(raw, lookAtState, dtSeconds);

      const weightedLookX = pointerX * pointerDeltaSensitivity;
      const weightedLookY = pointerY * pointerDeltaSensitivity;
      const nextApplied = runPipeline(raw, previousApplied, config, {
        kind: 'top-view',
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

    setFocus(position: ReadonlyVec3): void {
      vec3.set(focus, position[0], position[1], position[2]);
    },

    toZoom(weight?: number): void {
      zoomImpulse += weightOrDefault(weight);
    },

    setFocusLocked(locked: boolean): void {
      focusLocked = locked;
    },

    isFocusLocked(): boolean {
      return focusLocked;
    },
  };
}
