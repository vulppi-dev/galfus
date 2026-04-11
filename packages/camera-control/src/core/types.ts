import type { EntityId, World3DId } from '@vulfram/engine/world3d';
import type { Quat as quat, ReadonlyVec3, Vec3 as vec3 } from '@vulfram/engine/math';

export type CameraControllerKind =
  | 'orbit'
  | 'spectator'
  | 'first-person'
  | 'third-person'
  | 'top-view';

export interface CameraTarget {
  position: vec3;
  rotation: quat;
}

export interface CameraActionWeights {
  forward: number;
  right: number;
  up: number;
  zoom: number;
  lookX: number;
  lookY: number;
  lookAt: number;
}

export interface CameraPointerState {
  delta: vec3;
  lookDelta: vec3;
  leftPressed: boolean;
  middlePressed: boolean;
  rightPressed: boolean;
}

export interface CameraControllerContext {
  kind: CameraControllerKind;
  worldId: World3DId;
  cameraEntityId: EntityId;
  dtSeconds: number;
  pointer: CameraPointerState;
  weights: CameraActionWeights;
}

export type TranslationStrategy = (
  next: CameraTarget,
  prev: CameraTarget,
  context: CameraControllerContext
) => CameraTarget;

export type EasingFunction = (
  next: CameraTarget,
  prev: CameraTarget,
  context: CameraControllerContext
) => CameraTarget;

export interface CameraControllerOptions {
  translationStrategy?: TranslationStrategy;
  easing?: EasingFunction;
}

export interface BaseCameraControllerHandle {
  update(dtSeconds: number): void;
  lookAt(position: ReadonlyVec3, weight?: number): void;
}

export interface MotionCameraControllerHandle extends BaseCameraControllerHandle {
  pressForward(weight?: number): void;
  releaseForward(): void;
  pressBackward(weight?: number): void;
  releaseBackward(): void;
  pressLeft(weight?: number): void;
  releaseLeft(): void;
  pressRight(weight?: number): void;
  releaseRight(): void;
  pressUp(weight?: number): void;
  releaseUp(): void;
  pressDown(weight?: number): void;
  releaseDown(): void;
  toForward(weight?: number): void;
  toBackward(weight?: number): void;
  toLeft(weight?: number): void;
  toRight(weight?: number): void;
  toUp(weight?: number): void;
  toDown(weight?: number): void;
  look(deltaX: number, deltaY: number, weight?: number): void;
}

export interface LookAtState {
  enabled: boolean;
  target: vec3;
  weight: number;
}
