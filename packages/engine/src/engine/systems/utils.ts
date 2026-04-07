import { mat4, quat, vec2, vec3, vec4 } from 'gl-matrix';
import type { quat as Quat, vec2 as Vec2, vec3 as Vec3, vec4 as Vec4 } from 'gl-matrix';
import type { MaterialOptions, PbrOptions, StandardOptions } from '../../types/cmds/material';
import type {
  CubeOptions,
  PlaneOptions,
  PrimitiveOptions,
  PyramidOptions
} from '../../types/cmds/geometry';
import type { TransformComponent } from '../ecs';
import type { WorldState } from '../state';

/** Converts array-like numeric values into a fixed-length tuple buffer. */
export function toTuple(value: ArrayLike<number>, length: number): number[] {
  const result = new Array<number>(length);
  for (let i = 0; i < length; i++) {
    result[i] = Number(value[i] ?? 0);
  }
  return result;
}

/** Normalizes arbitrary 2D vector input to a strict tuple. */
export function toVec2(value: ArrayLike<number>): Vec2 {
  const result = toTuple(value, 2);
  return vec2.fromValues(result[0] ?? 0, result[1] ?? 0);
}

/** Normalizes arbitrary 3D vector input to a strict tuple. */
export function toVec3(value: ArrayLike<number>): Vec3 {
  const result = toTuple(value, 3);
  return vec3.fromValues(result[0] ?? 0, result[1] ?? 0, result[2] ?? 0);
}

/** Normalizes arbitrary 4D vector input to a strict tuple. */
export function toVec4(value: ArrayLike<number>): Vec4 {
  const result = toTuple(value, 4);
  return vec4.fromValues(result[0] ?? 0, result[1] ?? 0, result[2] ?? 0, result[3] ?? 0);
}

/** Normalizes arbitrary quaternion-like input to a strict `[x, y, z, w]` tuple. */
export function toQuat(value: ArrayLike<number>): Quat {
  const result = toTuple(value, 4);
  return quat.fromValues(result[0] ?? 0, result[1] ?? 0, result[2] ?? 0, result[3] ?? 1);
}

/** Normalizes standard-material option payload for command serialization. */
export function normalizeStandardOptions(options: StandardOptions): StandardOptions {
  const normalized: StandardOptions = { ...options };
  if (options.baseColor !== undefined) {
    normalized.baseColor = toVec4(options.baseColor);
  }
  if (options.emissiveColor !== undefined) {
    normalized.emissiveColor = options.emissiveColor ? toVec4(options.emissiveColor) : null;
  }
  if (options.specColor !== undefined) {
    normalized.specColor = options.specColor ? toVec4(options.specColor) : null;
  }
  if (options.toonParams !== undefined) {
    normalized.toonParams = options.toonParams ? toVec4(options.toonParams) : null;
  }
  return normalized;
}

/** Normalizes PBR-material option payload for command serialization. */
export function normalizePbrOptions(options: PbrOptions): PbrOptions {
  const normalized: PbrOptions = { ...options };
  if (options.baseColor !== undefined) {
    normalized.baseColor = toVec4(options.baseColor);
  }
  if (options.emissiveColor !== undefined) {
    normalized.emissiveColor = toVec4(options.emissiveColor);
  }
  return normalized;
}

/** Normalizes polymorphic material options into strict tuple-backed values. */
export function normalizeMaterialOptions(
  options: MaterialOptions | undefined
): MaterialOptions | undefined {
  if (!options) return options;
  if (options.type === 'standard') {
    return {
      type: 'standard',
      content: normalizeStandardOptions(options.content)
    };
  }
  return {
    type: 'pbr',
    content: normalizePbrOptions(options.content)
  };
}

/** Normalizes primitive options that contain vector payloads. */
export function normalizePrimitiveOptions(options: PrimitiveOptions): PrimitiveOptions {
  if (options.type === 'cube') {
    const content = options.content as CubeOptions;
    return {
      type: 'cube',
      content: {
        ...content,
        size: content.size ? toVec3(content.size) : content.size
      }
    };
  }
  if (options.type === 'plane') {
    const content = options.content as PlaneOptions;
    return {
      type: 'plane',
      content: {
        ...content,
        size: content.size ? toVec3(content.size) : content.size
      }
    };
  }
  if (options.type === 'pyramid') {
    const content = options.content as PyramidOptions;
    return {
      type: 'pyramid',
      content: {
        ...content,
        size: content.size ? toVec3(content.size) : content.size
      }
    };
  }
  return options;
}

/**
 * Resolves the local transform matrix for an entity, without applying constraints
 * (for example parent hierarchy composition).
 */
export function getEntityLocalTransformMatrix(world: WorldState, entityId: number): mat4 {
  const store = world.components.get(entityId);
  if (!store) return mat4.create();

  const transform = store.get('Transform') as TransformComponent | undefined;

  const m = mat4.create();
  if (transform) {
    const rotation = quat.fromValues(
      transform.rotation[0],
      transform.rotation[1],
      transform.rotation[2],
      transform.rotation[3]
    );
    const position = vec3.fromValues(
      transform.position[0],
      transform.position[1],
      transform.position[2]
    );
    const scale = vec3.fromValues(transform.scale[0], transform.scale[1], transform.scale[2]);
    mat4.fromRotationTranslationScale(m, rotation, position, scale);
  }
  return m;
}

/**
 * Returns the latest constraint-resolved world matrix for an entity.
 * Falls back to local transform when no resolved matrix is cached yet.
 */
export function getResolvedEntityTransformMatrix(world: WorldState, entityId: number): mat4 {
  const resolved = world.resolvedEntityTransforms.get(entityId);
  if (resolved && !hasDirtyConstraintPath(world, entityId)) {
    return mat4.clone(resolved);
  }

  // Fallback for same-frame reads before ConstraintSolve runs:
  // recompute parent-chain world transform directly from local components.
  return resolveEntityWorldTransformImmediate(world, entityId);
}

function hasDirtyConstraintPath(world: WorldState, entityId: number): boolean {
  let current: number | undefined = entityId;
  while (current !== undefined) {
    if (world.constraintDirtyEntities.has(current)) {
      return true;
    }
    current = world.constraintParentByChild.get(current);
  }
  return false;
}

function resolveEntityWorldTransformImmediate(world: WorldState, entityId: number): mat4 {
  const visiting = new Set<number>();

  const resolveRecursive = (currentId: number): mat4 => {
    if (visiting.has(currentId)) {
      return getEntityLocalTransformMatrix(world, currentId);
    }
    visiting.add(currentId);

    const local = getEntityLocalTransformMatrix(world, currentId);
    const parentId = world.constraintParentByChild.get(currentId);
    if (parentId === undefined) {
      visiting.delete(currentId);
      return local;
    }

    const parentWorld = resolveRecursive(parentId);
    const out = mat4.create();
    mat4.multiply(out, parentWorld, local);
    visiting.delete(currentId);
    return out;
  };

  return resolveRecursive(entityId);
}

/** Compares 4x4 matrices using an absolute epsilon per component. */
export function mat4EqualsApprox(
  a: ArrayLike<number>,
  b: ArrayLike<number>,
  epsilon = 1e-6
): boolean {
  for (let i = 0; i < 16; i++) {
    if (Math.abs((a[i] ?? 0) - (b[i] ?? 0)) > epsilon) {
      return false;
    }
  }
  return true;
}
