import type { EntityId, World3DId } from '@galfus/engine/world3d';
import type { Quat as quat, ReadonlyVec3, Vec3 as vec3 } from '@galfus/engine/math';

export type CameraControllerKind =
  | 'orbit'
  | 'spectator'
  | 'first-person'
  | 'third-person'
  | 'top-view';

/** World-space camera transform produced by a controller pipeline. */
export interface CameraTarget {
  position: vec3;
  rotation: quat;
}

/** Normalized action weights passed through the controller pipeline each frame. */
export interface CameraActionWeights {
  forward: number;
  right: number;
  up: number;
  zoom: number;
  lookX: number;
  lookY: number;
  lookAt: number;
}

/** Pointer snapshot consumed by camera controllers during `update()`. */
export interface CameraPointerState {
  delta: vec3;
  lookDelta: vec3;
  leftPressed: boolean;
  middlePressed: boolean;
  rightPressed: boolean;
}

/** Context passed into easing and translation strategies. */
export interface CameraControllerContext {
  kind: CameraControllerKind;
  worldId: World3DId;
  cameraEntityId: EntityId;
  dtSeconds: number;
  pointer: CameraPointerState;
  weights: CameraActionWeights;
}

/** Function used to reshape the raw target before easing is applied. */
export type TranslationStrategy = (
  next: CameraTarget,
  prev: CameraTarget,
  context: CameraControllerContext
) => CameraTarget;

/** Function used to smooth the final camera target before applying it to the world. */
export type EasingFunction = (
  next: CameraTarget,
  prev: CameraTarget,
  context: CameraControllerContext
) => CameraTarget;

/** Shared configuration options accepted by all controller factories. */
export interface CameraControllerOptions {
  translationStrategy?: TranslationStrategy;
  easing?: EasingFunction;
}

/** Minimal handle shared by every camera controller. */
export interface BaseCameraControllerHandle {
  update(dtSeconds: number): void;
  lookAt(position: ReadonlyVec3, weight?: number): void;
}

/** Extended handle for controllers that expose motion-style actions. */
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
