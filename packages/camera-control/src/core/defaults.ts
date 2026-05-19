import { quat, vec3 } from '@galfus/engine/math';
import { slerpArc } from './math';
import { createCameraTarget } from './pipeline';
import type {
  CameraControllerContext,
  CameraTarget,
  EasingFunction,
  TranslationStrategy
} from './types';

/**
 * Leaves the raw target untouched before easing.
 *
 * @example
 * ```ts
 * import { linearTranslationStrategy } from '@galfus/camera-control';
 *
 * const config = { translationStrategy: linearTranslationStrategy };
 * ```
 */
export const linearTranslationStrategy: TranslationStrategy = (next: CameraTarget): CameraTarget =>
  next;

/**
 * Leaves the post-translation target untouched.
 *
 * @example
 * ```ts
 * import { linearEasing } from '@galfus/camera-control';
 *
 * const config = { easing: linearEasing };
 * ```
 */
export const linearEasing: EasingFunction = (next: CameraTarget): CameraTarget => next;

/**
 * Creates an easing function that exponentially approaches the previous target.
 *
 * Higher values settle faster; lower values feel smoother and heavier.
 *
 * @example
 * ```ts
 * import { createExponentialEasing } from '@galfus/camera-control';
 *
 * const easing = createExponentialEasing(12);
 * ```
 */
export function createExponentialEasing(factorPerSecond = 10): EasingFunction {
  return (
    next: CameraTarget,
    prev: CameraTarget,
    context: CameraControllerContext
  ): CameraTarget => {
    const alpha = 1 - Math.exp(-factorPerSecond * context.dtSeconds);
    const out = createCameraTarget(prev.position, prev.rotation);
    vec3.lerp(out.position, prev.position, next.position, alpha);
    slerpArc(out.rotation, prev.rotation, next.rotation, alpha, false);
    return out;
  };
}

/**
 * Creates a translation strategy that scales the delta from `prev` to `next`.
 *
 * Use values between `0` and `1` for damped motion, values above `1` for overshoot,
 * and negative values to invert the translation arc.
 *
 * @example
 * ```ts
 * import { createWeightedTranslationStrategy } from '@galfus/camera-control';
 *
 * const translationStrategy = createWeightedTranslationStrategy(0.5);
 * ```
 */
export function createWeightedTranslationStrategy(scale = 1): TranslationStrategy {
  return (next: CameraTarget, prev: CameraTarget): CameraTarget => {
    const out = createCameraTarget(prev.position, prev.rotation);
    vec3.sub(out.position, next.position, prev.position);
    vec3.scale(out.position, out.position, scale);
    vec3.add(out.position, prev.position, out.position);

    const delta = quat.create();
    quat.mul(delta, next.rotation, quat.invert(delta, prev.rotation));
    const scaled = quat.create();
    slerpArc(scaled, [0, 0, 0, 1], delta, Math.min(1, Math.abs(scale)), scale < 0);
    quat.mul(out.rotation, scaled, prev.rotation);
    quat.normalize(out.rotation, out.rotation);
    return out;
  };
}
