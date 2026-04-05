import {
  mat4,
  vec3,
  vec4,
  type ReadonlyMat4,
  type ReadonlyVec2,
  type ReadonlyVec3,
  type vec3 as Vec3,
} from 'gl-matrix';

export interface Ray3 {
  origin: Vec3;
  direction: Vec3;
}

export interface RayHit {
  distance: number;
  point: Vec3;
}

export interface PointerRaycastInput {
  pointer: ReadonlyVec2;
  viewMatrix: ReadonlyMat4;
  projectionMatrix: ReadonlyMat4;
  viewportSize: ReadonlyVec2;
  viewportOrigin?: ReadonlyVec2;
}

export interface PointerRaycastWgpuInput extends PointerRaycastInput {}

/**
 * Builds a right-handed perspective projection in ZO depth range (0..1),
 * with optional reverse-Z by swapping near/far.
 *
 * Matches glam::Mat4::perspective_rh semantics used by core.
 */
export function createPerspectiveRhZo(
  fovYRadians: number,
  aspect: number,
  near: number,
  far: number,
): mat4 {
  const f = 1 / Math.tan(fovYRadians / 2);
  const out = mat4.create();

  out[0] = f / Math.max(1e-8, aspect);
  out[1] = 0;
  out[2] = 0;
  out[3] = 0;

  out[4] = 0;
  out[5] = f;
  out[6] = 0;
  out[7] = 0;

  out[8] = 0;
  out[9] = 0;
  out[10] = far / (near - far);
  out[11] = -1;

  out[12] = 0;
  out[13] = 0;
  out[14] = (near * far) / (near - far);
  out[15] = 0;

  return out;
}

export interface PointerEventRaycastData {
  position: ReadonlyVec2;
  positionTarget?: ReadonlyVec2;
  windowWidth?: number;
  windowHeight?: number;
  targetWidth?: number;
  targetHeight?: number;
}

export interface PointerEventRaycastInput {
  pointerEvent: PointerEventRaycastData;
  viewMatrix: ReadonlyMat4;
  projectionMatrix: ReadonlyMat4;
  fallbackViewportSize?: ReadonlyVec2;
  viewportOrigin?: ReadonlyVec2;
}

function pointOnRay(ray: Ray3, distance: number): Vec3 {
  return vec3.fromValues(
    ray.origin[0]! + ray.direction[0]! * distance,
    ray.origin[1]! + ray.direction[1]! * distance,
    ray.origin[2]! + ray.direction[2]! * distance,
  );
}

/**
 * Creates a world-space ray from pointer coordinates plus camera matrices.
 *
 * Returns `null` if the camera matrix cannot be inverted.
 */
export function createPointerRay(input: PointerRaycastInput): Ray3 | null {
  const pointerX = input.pointer[0]!;
  const pointerY = input.pointer[1]!;
  const viewportWidth = input.viewportSize[0]!;
  const viewportHeight = input.viewportSize[1]!;
  const viewportSource = input.viewportOrigin;
  const viewportX = viewportSource ? viewportSource[0]! : 0;
  const viewportY = viewportSource ? viewportSource[1]! : 0;

  if (viewportWidth <= 0 || viewportHeight <= 0) {
    return null;
  }

  const normalizedX = ((pointerX - viewportX) / viewportWidth) * 2 - 1;
  const normalizedY = 1 - ((pointerY - viewportY) / viewportHeight) * 2;

  const viewProjection = mat4.multiply(
    mat4.create(),
    input.projectionMatrix,
    input.viewMatrix,
  );
  const inverseViewProjection = mat4.invert(mat4.create(), viewProjection);
  if (!inverseViewProjection) {
    return null;
  }

  const nearClip = vec4.fromValues(normalizedX, normalizedY, -1, 1);
  const farClip = vec4.fromValues(normalizedX, normalizedY, 1, 1);

  const nearWorld4 = vec4.transformMat4(vec4.create(), nearClip, inverseViewProjection);
  const farWorld4 = vec4.transformMat4(vec4.create(), farClip, inverseViewProjection);
  if (nearWorld4[3] === 0 || farWorld4[3] === 0) {
    return null;
  }

  const nearWorld = vec3.fromValues(
    nearWorld4[0] / nearWorld4[3],
    nearWorld4[1] / nearWorld4[3],
    nearWorld4[2] / nearWorld4[3],
  );
  const farWorld = vec3.fromValues(
    farWorld4[0] / farWorld4[3],
    farWorld4[1] / farWorld4[3],
    farWorld4[2] / farWorld4[3],
  );

  const direction = vec3.normalize(
    vec3.create(),
    vec3.subtract(vec3.create(), farWorld, nearWorld),
  );

  return {
    origin: nearWorld,
    direction,
  };
}

/**
 * Creates a world-space ray using WGPU reverse-Z clip conventions.
 *
 * This mirrors core picking logic:
 * - NDC depth near=1 and far=0
 * - direction corrected to follow camera forward when needed
 */
export function createPointerRayWgpuReverseZ(
  input: PointerRaycastWgpuInput,
): Ray3 | null {
  const pointerX = input.pointer[0]!;
  const pointerY = input.pointer[1]!;
  const viewportWidth = input.viewportSize[0]!;
  const viewportHeight = input.viewportSize[1]!;
  const viewportSource = input.viewportOrigin;
  const viewportX = viewportSource ? viewportSource[0]! : 0;
  const viewportY = viewportSource ? viewportSource[1]! : 0;

  if (viewportWidth <= 0 || viewportHeight <= 0) {
    return null;
  }

  const normalizedX = ((pointerX - viewportX) / viewportWidth) * 2 - 1;
  const normalizedY = 1 - ((pointerY - viewportY) / viewportHeight) * 2;

  const viewProjection = mat4.multiply(
    mat4.create(),
    input.projectionMatrix,
    input.viewMatrix,
  );
  const inverseViewProjection = mat4.invert(mat4.create(), viewProjection);
  if (!inverseViewProjection) {
    return null;
  }

  const nearClip = vec4.fromValues(normalizedX, normalizedY, 1, 1);
  const farClip = vec4.fromValues(normalizedX, normalizedY, 0, 1);

  const nearWorld4 = vec4.transformMat4(vec4.create(), nearClip, inverseViewProjection);
  const farWorld4 = vec4.transformMat4(vec4.create(), farClip, inverseViewProjection);
  if (nearWorld4[3] === 0 || farWorld4[3] === 0) {
    return null;
  }

  const nearWorld = vec3.fromValues(
    nearWorld4[0] / nearWorld4[3],
    nearWorld4[1] / nearWorld4[3],
    nearWorld4[2] / nearWorld4[3],
  );
  const farWorld = vec3.fromValues(
    farWorld4[0] / farWorld4[3],
    farWorld4[1] / farWorld4[3],
    farWorld4[2] / farWorld4[3],
  );

  let direction = vec3.normalize(
    vec3.create(),
    vec3.subtract(vec3.create(), farWorld, nearWorld),
  );

  const inverseView = mat4.invert(mat4.create(), input.viewMatrix);
  if (!inverseView) {
    return null;
  }
  const origin4 = vec4.transformMat4(
    vec4.create(),
    vec4.fromValues(0, 0, 0, 1),
    inverseView,
  );
  if (origin4[3] === 0) {
    return null;
  }
  const origin = vec3.fromValues(
    origin4[0] / origin4[3],
    origin4[1] / origin4[3],
    origin4[2] / origin4[3],
  );

  const forward4 = vec4.transformMat4(
    vec4.create(),
    vec4.fromValues(0, 0, -1, 0),
    inverseView,
  );
  const cameraForward = vec3.normalize(
    vec3.create(),
    vec3.fromValues(forward4[0], forward4[1], forward4[2]),
  );
  if (vec3.squaredLength(cameraForward) > 1e-8 && vec3.dot(direction, cameraForward) < 0) {
    direction = vec3.scale(vec3.create(), direction, -1);
  }

  if (vec3.squaredLength(direction) <= 1e-8) {
    return null;
  }

  return { origin, direction };
}

/**
 * Creates a world-space ray using pointer event dimensions from core.
 *
 * Priority:
 * 1) target-space pointer + target size (most accurate for routed targets)
 * 2) window-space pointer + window size
 * 3) fallback viewport size (if provided)
 *
 * Uses WGPU reverse-Z conventions to match core picking.
 */
export function createPointerRayFromEvent(
  input: PointerEventRaycastInput,
): Ray3 | null {
  const event = input.pointerEvent;
  const targetWidth = event.targetWidth;
  const targetHeight = event.targetHeight;
  const windowWidth = event.windowWidth;
  const windowHeight = event.windowHeight;

  if (
    event.positionTarget &&
    typeof targetWidth === 'number' &&
    typeof targetHeight === 'number' &&
    targetWidth > 0 &&
    targetHeight > 0
  ) {
    return createPointerRayWgpuReverseZ({
      pointer: event.positionTarget,
      viewMatrix: input.viewMatrix,
      projectionMatrix: input.projectionMatrix,
      viewportSize: [targetWidth, targetHeight],
      viewportOrigin: [0, 0],
    });
  }

  if (
    typeof windowWidth === 'number' &&
    typeof windowHeight === 'number' &&
    windowWidth > 0 &&
    windowHeight > 0
  ) {
    return createPointerRayWgpuReverseZ({
      pointer: event.position,
      viewMatrix: input.viewMatrix,
      projectionMatrix: input.projectionMatrix,
      viewportSize: [windowWidth, windowHeight],
      viewportOrigin: input.viewportOrigin,
    });
  }

  if (input.fallbackViewportSize) {
    return createPointerRayWgpuReverseZ({
      pointer: event.position,
      viewMatrix: input.viewMatrix,
      projectionMatrix: input.projectionMatrix,
      viewportSize: input.fallbackViewportSize,
      viewportOrigin: input.viewportOrigin,
    });
  }

  return null;
}

/**
 * Intersects a ray against a plane defined by one point and one normal.
 *
 * Returns `null` when the ray is parallel to the plane or the hit is behind the ray.
 */
export function intersectRayPlane(
  ray: Ray3,
  planePoint: ReadonlyVec3,
  planeNormal: ReadonlyVec3,
): RayHit | null {
  const origin = vec3.fromValues(ray.origin[0], ray.origin[1], ray.origin[2]);
  const direction = vec3.fromValues(ray.direction[0], ray.direction[1], ray.direction[2]);
  const normal = vec3.normalize(
    vec3.create(),
    vec3.fromValues(planeNormal[0], planeNormal[1], planeNormal[2]),
  );
  const point = vec3.fromValues(planePoint[0], planePoint[1], planePoint[2]);

  const denominator = vec3.dot(direction, normal);
  if (Math.abs(denominator) < 1e-6) {
    return null;
  }

  const distance =
    vec3.dot(vec3.subtract(vec3.create(), point, origin), normal) / denominator;
  if (distance < 0) {
    return null;
  }

  return {
    distance,
    point: pointOnRay(ray, distance),
  };
}

/**
 * Intersects a ray with a sphere and returns the closest hit in front of the ray origin.
 */
export function intersectRaySphere(
  ray: Ray3,
  center: ReadonlyVec3,
  radius: number,
): RayHit | null {
  const origin = vec3.fromValues(ray.origin[0], ray.origin[1], ray.origin[2]);
  const direction = vec3.fromValues(ray.direction[0], ray.direction[1], ray.direction[2]);
  const sphereCenter = vec3.fromValues(center[0], center[1], center[2]);

  const oc = vec3.subtract(vec3.create(), origin, sphereCenter);
  const a = vec3.dot(direction, direction);
  const b = 2 * vec3.dot(oc, direction);
  const c = vec3.dot(oc, oc) - radius * radius;
  const discriminant = b * b - 4 * a * c;

  if (discriminant < 0) {
    return null;
  }

  const sqrtDiscriminant = Math.sqrt(discriminant);
  const t0 = (-b - sqrtDiscriminant) / (2 * a);
  const t1 = (-b + sqrtDiscriminant) / (2 * a);

  const distance = t0 >= 0 ? t0 : t1 >= 0 ? t1 : -1;
  if (distance < 0) {
    return null;
  }

  return {
    distance,
    point: pointOnRay(ray, distance),
  };
}

/**
 * Intersects a ray with an axis-aligned bounding box.
 */
export function intersectRayAabb(
  ray: Ray3,
  min: ReadonlyVec3,
  max: ReadonlyVec3,
): RayHit | null {
  const origin = ray.origin;
  const direction = ray.direction;

  let tMin = -Infinity;
  let tMax = Infinity;

  const axes = [0, 1, 2] as const;
  for (const axis of axes) {
    const o = origin[axis]!;
    const d = direction[axis]!;
    const minAxis = min[axis]!;
    const maxAxis = max[axis]!;

    if (Math.abs(d) < 1e-8) {
      if (o < minAxis || o > maxAxis) {
        return null;
      }
      continue;
    }

    const invD = 1 / d;
    let t0 = (minAxis - o) * invD;
    let t1 = (maxAxis - o) * invD;
    if (t0 > t1) {
      const tmp = t0;
      t0 = t1;
      t1 = tmp;
    }

    tMin = Math.max(tMin, t0);
    tMax = Math.min(tMax, t1);
    if (tMax < tMin) {
      return null;
    }
  }

  const distance = tMin >= 0 ? tMin : tMax >= 0 ? tMax : -1;
  if (distance < 0) {
    return null;
  }

  return {
    distance,
    point: pointOnRay(ray, distance),
  };
}
