import type { Node, Scene } from '@gltf-transform/core';
import type { Quat, Vec3 } from '@galfus/engine/math';
import {
  create3DEntity,
  create3DModel,
  remove3DEntity,
  set3DParent,
  update3DTransform,
  type EntityId,
  type World3DId
} from '@galfus/engine/world3d';
import { toQuat, toVec3 } from './convert';
import { ensureMaterial, ensurePrimitiveGeometry } from './resources';
import type { GltfInstance, GltfInstantiateOptions, LoaderContext, SceneTemplate } from './types';

function isIdentityVec3(value: Vec3, identity: number): boolean {
  return value[0] === identity && value[1] === identity && value[2] === identity;
}

function isIdentityQuat(value: Quat): boolean {
  return value[0] === 0 && value[1] === 0 && value[2] === 0 && value[3] === 1;
}

function createNodeTemplate(ctx: LoaderContext, node: Node, nodes: SceneTemplate['nodes']): number {
  const mesh = node.getMesh();
  const primitives: SceneTemplate['nodes'][number]['primitives'] = [];

  if (mesh) {
    for (const primitive of mesh.listPrimitives()) {
      const geometryId = ensurePrimitiveGeometry(ctx, primitive);
      if (!geometryId) continue;
      const materialId = ensureMaterial(ctx, primitive.getMaterial());
      primitives.push({ geometryId, materialId });
    }
  }

  const nodeIndex = nodes.length;
  nodes.push({
    name: node.getName() || undefined,
    translation: toVec3(node.getTranslation()),
    rotation: toQuat(node.getRotation(), 1),
    scale: toVec3(node.getScale()),
    children: [],
    primitives
  });

  for (const child of node.listChildren()) {
    const childIndex = createNodeTemplate(ctx, child, nodes);
    nodes[nodeIndex]!.children.push(childIndex);
  }

  return nodeIndex;
}

/** Builds an immutable scene template and uploads all required resources. */
export function buildSceneTemplate(ctx: LoaderContext, scene: Scene): SceneTemplate {
  const nodes: SceneTemplate['nodes'] = [];
  const roots: number[] = [];

  for (const node of scene.listChildren()) {
    roots.push(createNodeTemplate(ctx, node, nodes));
  }

  return { roots, nodes };
}

function setInstanceRootTransform(
  worldId: World3DId,
  rootEntityId: EntityId,
  options: GltfInstantiateOptions | undefined
): void {
  const rootTransform = options?.rootTransform;
  if (!rootTransform) return;
  update3DTransform(worldId, rootEntityId, rootTransform);
}

/** Instantiates entities/models/parents from a previously built scene template. */
export function instantiateTemplate(
  worldId: World3DId,
  template: SceneTemplate,
  options?: GltfInstantiateOptions
): GltfInstance {
  const entityIds: EntityId[] = [];
  let disposed = false;

  const rootEntityId = create3DEntity(worldId);
  entityIds.push(rootEntityId);
  setInstanceRootTransform(worldId, rootEntityId, options);

  const nodeEntityIds: EntityId[] = new Array(template.nodes.length);

  for (let i = 0; i < template.nodes.length; i++) {
    const node = template.nodes[i];
    if (!node) continue;

    const nodeEntityId = create3DEntity(worldId);
    nodeEntityIds[i] = nodeEntityId;
    entityIds.push(nodeEntityId);

    const transformPatch = {
      position: isIdentityVec3(node.translation, 0) ? undefined : node.translation,
      rotation: isIdentityQuat(node.rotation) ? undefined : node.rotation,
      scale: isIdentityVec3(node.scale, 1) ? undefined : node.scale
    };
    update3DTransform(worldId, nodeEntityId, transformPatch);
  }

  for (const rootIndex of template.roots) {
    const rootNodeEntity = nodeEntityIds[rootIndex];
    if (rootNodeEntity === undefined) continue;
    set3DParent(worldId, rootNodeEntity, rootEntityId);
  }

  for (let i = 0; i < template.nodes.length; i++) {
    const node = template.nodes[i];
    const parentEntity = nodeEntityIds[i];
    if (!node || parentEntity === undefined) continue;

    for (const childIndex of node.children) {
      const childEntity = nodeEntityIds[childIndex];
      if (childEntity === undefined) continue;
      set3DParent(worldId, childEntity, parentEntity);
    }

    if (node.primitives.length === 0) continue;

    if (node.primitives.length === 1) {
      const primitive = node.primitives[0]!;
      create3DModel(worldId, parentEntity, {
        geometryId: primitive.geometryId,
        materialId: primitive.materialId
      });
      continue;
    }

    for (const primitive of node.primitives) {
      const modelEntity = create3DEntity(worldId);
      entityIds.push(modelEntity);
      set3DParent(worldId, modelEntity, parentEntity);
      create3DModel(worldId, modelEntity, {
        geometryId: primitive.geometryId,
        materialId: primitive.materialId
      });
    }
  }

  return {
    rootEntityId,
    entityIds,
    disposeEntities() {
      if (disposed) return;
      disposed = true;
      for (let i = entityIds.length - 1; i >= 0; i--) {
        const entityId = entityIds[i];
        if (entityId !== undefined) {
          remove3DEntity(worldId, entityId);
        }
      }
    }
  };
}
