import type { Accessor, Material, Node, Primitive, Texture } from '@gltf-transform/core';
import type { Quat as quat, Vec3 as vec3 } from '@galfus/engine/math';
import type {
  EntityId,
  GeometryId,
  MaterialId,
  TextureId,
  World3DId
} from '@galfus/engine/world3d';

/** Supported source formats for glTF scene loading. */
export type GltfSourceFormat = 'gltf' | 'glb';

/** Binary-like payload accepted by the loader. */
export type BinaryLike = Uint8Array | ArrayBuffer | ArrayBufferView;

/** Optional root transform applied to the imported scene root entity. */
export type RootTransform = {
  position?: vec3;
  rotation?: quat;
  scale?: vec3;
};

/** Instantiation options for a loaded glTF asset template. */
export interface GltfInstantiateOptions {
  rootTransform?: RootTransform;
}

/**
 * One instantiated scene graph from a loaded glTF asset.
 *
 * The instance owns only its created entities. Shared resources remain owned by
 * the parent `LoadedGltfAsset` until `disposeAll()` is called there.
 */
export interface GltfInstance {
  rootEntityId: EntityId;
  entityIds: EntityId[];
  disposeEntities(): void;
}

/** Static template node with local transform and mesh primitive bindings. */
export interface NodeTemplate {
  name?: string;
  translation: vec3;
  rotation: quat;
  scale: vec3;
  children: number[];
  primitives: Array<{
    geometryId: GeometryId;
    materialId: MaterialId;
  }>;
}

/** Immutable scene template produced from glTF document parsing. */
export interface SceneTemplate {
  roots: number[];
  nodes: NodeTemplate[];
}

/** Resource IDs allocated for one loaded glTF asset. */
export interface LoadedResourceIds {
  geometries: GeometryId[];
  materials: MaterialId[];
  textures: TextureId[];
}

/**
 * Loaded glTF asset with reusable resources and instantiation API.
 *
 * Use `instantiate()` to create one or more entity graphs, `disposeEntities()`
 * to remove only active instances, and `disposeAll()` to also release the shared
 * textures, materials, and geometries created for this asset.
 */
export interface LoadedGltfAsset {
  worldId: World3DId;
  warnings: string[];
  template: SceneTemplate;
  resources: LoadedResourceIds;
  instantiate(options?: GltfInstantiateOptions): GltfInstance;
  disposeEntities(): void;
  disposeAll(): void;
}

/**
 * Input descriptor for glTF/GLB loading.
 *
 * `data` is the main `.glb` or `.gltf` payload. When loading external-buffer or
 * external-texture `.gltf` files, provide the extra file contents in `resources`
 * keyed by their URI.
 */
export interface GltfLoadInput {
  worldId: World3DId;
  data: BinaryLike;
  format?: GltfSourceFormat;
  materialMode?: 'pbr' | 'standard';
  resources?: Record<string, BinaryLike>;
  rootTransform?: RootTransform;
  labelPrefix?: string;
}

/** Result summary for the legacy one-shot `loadGltfScene()` API. */
export interface GltfLoadResult {
  rootEntityId: EntityId;
  entityCount: number;
  geometryCount: number;
  materialCount: number;
  textureCount: number;
  warnings: string[];
}

/** Stable loader error codes for caller handling. */
export type GltfLoaderErrorCode =
  | 'INVALID_INPUT'
  | 'UNSUPPORTED_FORMAT'
  | 'PARSE_ERROR'
  | 'UNSUPPORTED_FEATURE'
  | 'MISSING_RESOURCE';

export type LoaderCounters = {
  entities: number;
  geometries: number;
  materials: number;
  textures: number;
};

export type LoaderContext = {
  worldId: World3DId;
  warnings: string[];
  labelPrefix: string;
  materialMode: 'pbr' | 'standard';
  defaultMaterialId?: MaterialId;
  geometryByPrimitive: WeakMap<Primitive, GeometryId>;
  materialBySource: WeakMap<Material, MaterialId>;
  textureBySource: WeakMap<Texture, TextureId>;
  textureColorSpaceHint: WeakMap<Texture, boolean>;
  uploadedVertexByAccessor: WeakMap<Accessor, number>;
  uploadedIndexByAccessor: WeakMap<Accessor, number>;
  createdGeometryIds: Set<GeometryId>;
  createdMaterialIds: Set<MaterialId>;
  createdTextureIds: Set<TextureId>;
  counters: LoaderCounters;
};
