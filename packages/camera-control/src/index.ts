export {
  createExponentialEasing,
  createWeightedTranslationStrategy,
  linearEasing,
  linearTranslationStrategy
} from './core/defaults';

export type {
  BaseCameraControllerHandle,
  CameraActionWeights,
  CameraControllerContext,
  CameraControllerKind,
  CameraControllerOptions,
  CameraPointerState,
  CameraTarget,
  EasingFunction,
  MotionCameraControllerHandle,
  TranslationStrategy
} from './core/types';

export {
  createOrbitController,
  type OrbitControllerConfig,
  type OrbitControllerHandle
} from './controllers/orbit';

export { createSpectatorController, type SpectatorControllerConfig } from './controllers/spectator';

export {
  createFirstPersonController,
  type FirstPersonControllerConfig
} from './controllers/first-person';

export {
  createThirdPersonController,
  type ThirdPersonControllerConfig,
  type ThirdPersonControllerHandle
} from './controllers/third-person';

export {
  createTopViewController,
  type TopViewControllerConfig,
  type TopViewControllerHandle
} from './controllers/top-view';
