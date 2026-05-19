import { uploadBuffer } from '@galfus/engine/core';
import { BUFFER_ID_BASE } from './constants';
import type { GltfLoadInput, LoaderContext } from './types';

let nextUploadBufferId = BUFFER_ID_BASE;

/** Creates mutable loader context for one import operation. */
export function createContext(input: GltfLoadInput): LoaderContext {
  return {
    worldId: input.worldId,
    warnings: [],
    labelPrefix: input.labelPrefix ?? 'gltf',
    materialMode: input.materialMode ?? 'pbr',
    defaultMaterialId: undefined,
    geometryByPrimitive: new WeakMap(),
    materialBySource: new WeakMap(),
    textureBySource: new WeakMap(),
    textureColorSpaceHint: new WeakMap(),
    uploadedVertexByAccessor: new WeakMap(),
    uploadedIndexByAccessor: new WeakMap(),
    createdGeometryIds: new Set(),
    createdMaterialIds: new Set(),
    createdTextureIds: new Set(),
    counters: {
      entities: 0,
      geometries: 0,
      materials: 0,
      textures: 0
    }
  };
}

function allocBufferId(): number {
  const id = nextUploadBufferId;
  nextUploadBufferId += 1;
  return id;
}

/** Uploads bytes to engine buffer system and returns allocated buffer id. */
export function uploadBytes(
  _ctx: LoaderContext,
  usage: 'image-data' | 'vertex-data' | 'index-data',
  bytes: Uint8Array
): number {
  const bufferId = allocBufferId();
  uploadBuffer(bufferId, usage, bytes);
  return bufferId;
}
