import {
  dispose3DGeometry,
  dispose3DMaterial,
  dispose3DTexture,
} from '@vulfram/engine/world3d';
import { createContext } from './context';
import { GltfLoaderError } from './errors';
import { readDocument } from './parse';
import { buildSceneTemplate, instantiateTemplate } from './scene';
import type {
  GltfInstantiateOptions,
  GltfLoadInput,
  GltfLoadResult,
  GltfInstance,
  LoadedGltfAsset,
} from './types';

export type {
  BinaryLike,
  GltfInstantiateOptions,
  GltfInstance,
  GltfLoadInput,
  GltfLoadResult,
  GltfLoaderErrorCode,
  LoadedGltfAsset,
  LoadedResourceIds,
  NodeTemplate,
  SceneTemplate,
  GltfSourceFormat,
  RootTransform,
} from './types';
export { GltfLoaderError } from './errors';

/**
 * Loads glTF/GLB resources and returns a reusable asset template.
 *
 * The returned asset can instantiate entity graphs multiple times, reusing
 * uploaded textures/materials/geometries across instances.
 */
export async function loadGltfAsset(input: GltfLoadInput): Promise<LoadedGltfAsset> {
  const document = await readDocument(input);
  const ctx = createContext(input);

  const root = document.getRoot();
  const scene = root.getDefaultScene() ?? root.listScenes()[0] ?? null;
  if (!scene) {
    throw new GltfLoaderError('PARSE_ERROR', 'Document has no scene to import.');
  }

  const template = buildSceneTemplate(ctx, scene);
  const instances = new Set<GltfInstance>();
  let disposedAll = false;

  const disposeEntities = () => {
    for (const instance of [...instances]) {
      instance.disposeEntities();
      instances.delete(instance);
    }
  };

  const disposeAll = () => {
    if (disposedAll) return;
    disposedAll = true;
    disposeEntities();

    for (const geometryId of ctx.createdGeometryIds) {
      dispose3DGeometry(input.worldId, geometryId);
    }
    for (const materialId of ctx.createdMaterialIds) {
      dispose3DMaterial(input.worldId, materialId);
    }
    for (const textureId of ctx.createdTextureIds) {
      dispose3DTexture(input.worldId, textureId);
    }
  };

  const instantiate = (options?: GltfInstantiateOptions): GltfInstance => {
    if (disposedAll) {
      throw new GltfLoaderError(
        'INVALID_INPUT',
        'Cannot instantiate from a disposed glTF asset. Load it again.',
      );
    }

    const instance = instantiateTemplate(input.worldId, template, options);
    const originalDispose = instance.disposeEntities;
    instance.disposeEntities = () => {
      originalDispose();
      instances.delete(instance);
    };
    instances.add(instance);
    return instance;
  };

  return {
    worldId: input.worldId,
    warnings: ctx.warnings,
    template,
    resources: {
      geometries: [...ctx.createdGeometryIds],
      materials: [...ctx.createdMaterialIds],
      textures: [...ctx.createdTextureIds],
    },
    instantiate,
    disposeEntities,
    disposeAll,
  };
}

/**
 * Legacy one-shot API.
 *
 * Loads resources and immediately instantiates one entity graph.
 */
export async function loadGltfScene(input: GltfLoadInput): Promise<GltfLoadResult> {
  const asset = await loadGltfAsset(input);
  const instance = asset.instantiate({
    rootTransform: input.rootTransform,
  });

  return {
    rootEntityId: instance.rootEntityId,
    entityCount: instance.entityIds.length,
    geometryCount: asset.resources.geometries.length,
    materialCount: asset.resources.materials.length,
    textureCount: asset.resources.textures.length,
    warnings: asset.warnings,
  };
}
