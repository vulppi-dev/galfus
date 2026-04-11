import { Accessor, TextureInfo } from '@gltf-transform/core';
import { quat, vec3, vec4, type Quat, type Vec3, type Vec4 } from '@vulfram/engine/math';
import type { GeometryPrimitiveEntry, SamplerMode } from '@vulfram/engine/types';
import { U16_MAX } from './constants';
import { GltfLoaderError } from './errors';

export function toVec3(v: ArrayLike<number>): Vec3 {
  return vec3.fromValues(v[0] ?? 0, v[1] ?? 0, v[2] ?? 0);
}

export function toVec4(v: ArrayLike<number>, fallbackW = 1): Vec4 {
  return vec4.fromValues(v[0] ?? 0, v[1] ?? 0, v[2] ?? 0, v[3] ?? fallbackW);
}

export function toQuat(v: ArrayLike<number>, fallbackW = 1): Quat {
  return quat.fromValues(v[0] ?? 0, v[1] ?? 0, v[2] ?? 0, v[3] ?? fallbackW);
}

export function semanticToPrimitiveType(
  semantic: string
): GeometryPrimitiveEntry['primitiveType'] | null {
  switch (semantic) {
    case 'POSITION':
      return 'position';
    case 'NORMAL':
      return 'normal';
    case 'TANGENT':
      return 'tangent';
    case 'COLOR_0':
      return 'color';
    case 'TEXCOORD_0':
    case 'TEXCOORD_1':
      return 'u-v';
    case 'JOINTS_0':
      return 'skin-joints';
    case 'WEIGHTS_0':
      return 'skin-weights';
    default:
      return null;
  }
}

export function samplerFromTextureInfo(
  info: {
    getWrapS(): number;
    getWrapT(): number;
    getMagFilter(): number | null;
    getMinFilter(): number | null;
  } | null
): SamplerMode {
  const WRAP_REPEAT = TextureInfo.WrapMode.REPEAT;
  const WRAP_MIRRORED_REPEAT = TextureInfo.WrapMode.MIRRORED_REPEAT;
  const MAG_LINEAR = TextureInfo.MagFilter.LINEAR;
  const MIN_LINEAR = TextureInfo.MinFilter.LINEAR;
  const MIN_LINEAR_MIPMAP_LINEAR = TextureInfo.MinFilter.LINEAR_MIPMAP_LINEAR;
  const MIN_LINEAR_MIPMAP_NEAREST = TextureInfo.MinFilter.LINEAR_MIPMAP_NEAREST;

  const wrapS = info?.getWrapS() ?? WRAP_REPEAT;
  const wrapT = info?.getWrapT() ?? WRAP_REPEAT;
  const mag = info?.getMagFilter();
  const min = info?.getMinFilter();

  const repeat =
    wrapS === WRAP_REPEAT ||
    wrapS === WRAP_MIRRORED_REPEAT ||
    wrapT === WRAP_REPEAT ||
    wrapT === WRAP_MIRRORED_REPEAT;

  const linear =
    mag === null ||
    mag === MAG_LINEAR ||
    min === null ||
    min === MIN_LINEAR ||
    min === MIN_LINEAR_MIPMAP_LINEAR ||
    min === MIN_LINEAR_MIPMAP_NEAREST;

  if (repeat) return linear ? 'linear-repeat' : 'point-repeat';
  return linear ? 'linear-clamp' : 'point-clamp';
}

export function asBytes(array: ArrayBufferView): Uint8Array {
  return new Uint8Array(array.buffer, array.byteOffset, array.byteLength);
}

function toFloatArray(accessor: Accessor, components: number, fillTail = 0): Float32Array {
  const count = accessor.getCount();
  const srcArray = accessor.getArray();
  const elementSize = accessor.getElementSize();

  if (srcArray instanceof Float32Array && !accessor.getNormalized() && elementSize === components) {
    return srcArray;
  }

  const out = new Float32Array(count * components);
  const tmp: number[] = [];
  for (let i = 0; i < count; i++) {
    accessor.getElement(i, tmp);
    for (let c = 0; c < components; c++) {
      const v = tmp[c];
      out[i * components + c] = v ?? (c === 3 ? 1 : fillTail);
    }
  }
  return out;
}

function toUint16Vec4(accessor: Accessor): Uint16Array {
  const count = accessor.getCount();
  const out = new Uint16Array(count * 4);
  const srcArray = accessor.getArray();

  if (srcArray instanceof Uint16Array && accessor.getElementSize() === 4) {
    return srcArray;
  }

  const tmp: number[] = [];
  for (let i = 0; i < count; i++) {
    accessor.getElement(i, tmp);
    out[i * 4 + 0] = Math.max(0, Math.min(U16_MAX, Math.round(tmp[0] ?? 0)));
    out[i * 4 + 1] = Math.max(0, Math.min(U16_MAX, Math.round(tmp[1] ?? 0)));
    out[i * 4 + 2] = Math.max(0, Math.min(U16_MAX, Math.round(tmp[2] ?? 0)));
    out[i * 4 + 3] = Math.max(0, Math.min(U16_MAX, Math.round(tmp[3] ?? 0)));
  }
  return out;
}

function toUint32Indices(accessor: Accessor): Uint32Array {
  const src = accessor.getArray();
  if (!src) {
    throw new GltfLoaderError('PARSE_ERROR', 'Index accessor has no backing array.');
  }
  if (src instanceof Uint32Array) {
    return src;
  }
  const out = new Uint32Array(src.length);
  for (let i = 0; i < src.length; i++) {
    out[i] = src[i] ?? 0;
  }
  return out;
}

/** Converts accessor data to core-compatible stream bytes by semantic. */
export function accessorToStreamBytes(accessor: Accessor, semantic: string): Uint8Array {
  if (semantic === 'POSITION' || semantic === 'NORMAL') {
    return asBytes(toFloatArray(accessor, 3));
  }
  if (semantic === 'TANGENT') {
    return asBytes(toFloatArray(accessor, 4));
  }
  if (semantic === 'COLOR_0') {
    return asBytes(toFloatArray(accessor, 4, 1));
  }
  if (semantic === 'TEXCOORD_0' || semantic === 'TEXCOORD_1') {
    return asBytes(toFloatArray(accessor, 2));
  }
  if (semantic === 'JOINTS_0') {
    return asBytes(toUint16Vec4(accessor));
  }
  if (semantic === 'WEIGHTS_0') {
    return asBytes(toFloatArray(accessor, 4));
  }

  throw new GltfLoaderError('UNSUPPORTED_FEATURE', `Unsupported accessor semantic: ${semantic}`);
}

/** Converts glTF index accessor to u32 index stream bytes required by core. */
export function accessorToIndexBytes(accessor: Accessor): Uint8Array {
  return asBytes(toUint32Indices(accessor));
}
