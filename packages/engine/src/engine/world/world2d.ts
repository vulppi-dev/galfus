import { createWorld2D as createWorld2DRaw } from '../api';
import type {
  CameraProps,
  GeometryProps,
  MaterialProps,
  TagProps,
  TextureProps,
  TransformProps
} from '../ecs';
import type {
  CmdRealmRenderGraphBindArgs,
  CmdRenderGraphDisposeArgs,
  CmdRenderGraphListArgs,
  CmdRenderGraphUpsertArgs
} from '../../types/cmds/render-graph';
import type {
  CmdMaterialDefinitionDisposeArgs,
  CmdMaterialDefinitionUpsertArgs,
  CmdMaterialInstanceDisposeArgs,
  CmdMaterialInstanceUpsertArgs
} from '../../types/cmds/material';
import type { CmdTargetMeasurementArgs } from '../../types/cmds/target';
import type { NotificationLevel } from '../../types/kinds';
import {
  bindRealmRenderGraph as bindRealmRenderGraphRaw,
  createCamera as createCameraRaw,
  createEntity as createEntityRaw,
  createGeometry as createGeometryRaw,
  createMaterial as createMaterialRaw,
  createShape2D as createShape2DRaw,
  createSprite2D as createSprite2DRaw,
  createTag as createTagRaw,
  createTexture as createTextureRaw,
  disposeGeometry as disposeGeometryRaw,
  disposeMaterial as disposeMaterialRaw,
  disposeMaterialDefinition as disposeMaterialDefinitionRaw,
  disposeMaterialInstance as disposeMaterialInstanceRaw,
  disposeRenderGraph as disposeRenderGraphRaw,
  disposeTexture as disposeTextureRaw,
  getWorldRealmId as getWorldRealmIdRaw,
  listCameras as listCamerasRaw,
  listGeometries as listGeometriesRaw,
  listMaterials as listMaterialsRaw,
  listRenderGraphs as listRenderGraphsRaw,
  listTextures as listTexturesRaw,
  measureTarget as measureTargetRaw,
  removeEntity as removeEntityRaw,
  sendNotification as sendNotificationRaw,
  upsertRenderGraph as upsertRenderGraphRaw,
  upsertMaterialDefinition as upsertMaterialDefinitionRaw,
  upsertMaterialInstance as upsertMaterialInstanceRaw,
  updateTransform as updateTransformRaw
} from './entities';
import type {
  CommandId,
  EntityId,
  GeometryId,
  MaterialId,
  TextureId,
  World2DId
} from './types';
import {
  asCommandId,
  asEntityId,
  asGeometryId,
  asMaterialId,
  asTextureId,
  asWorld2DId,
  asWorldNumber
} from './types';

export type Create2DWorldOptions = {
  importance?: number;
  cachePolicy?: number;
  flags?: number;
};

export function create2DWorld(options?: Create2DWorldOptions): World2DId {
  return asWorld2DId(createWorld2DRaw(options));
}

export function create2DEntity(worldId: World2DId): EntityId {
  return asEntityId(createEntityRaw(asWorldNumber(worldId)));
}

export function remove2DEntity(worldId: World2DId, entityId: EntityId): void {
  removeEntityRaw(asWorldNumber(worldId), entityId as number);
}

export function update2DTransform(
  worldId: World2DId,
  entityId: EntityId,
  props: TransformProps
): void {
  updateTransformRaw(asWorldNumber(worldId), entityId as number, props);
}

export function create2DTag(worldId: World2DId, entityId: EntityId, props: TagProps): void {
  createTagRaw(asWorldNumber(worldId), entityId as number, props);
}

export function create2DCamera(worldId: World2DId, entityId: EntityId, props: CameraProps): void {
  createCameraRaw(asWorldNumber(worldId), entityId as number, {
    kind: 'orthographic',
    ...props
  });
}

export function create2DMaterial(worldId: World2DId, props: MaterialProps): MaterialId {
  return asMaterialId(createMaterialRaw(asWorldNumber(worldId), props));
}

export function create2DGeometry(worldId: World2DId, props: GeometryProps): GeometryId {
  return asGeometryId(createGeometryRaw(asWorldNumber(worldId), props));
}

export function create2DTexture(worldId: World2DId, props: TextureProps): TextureId {
  return asTextureId(createTextureRaw(asWorldNumber(worldId), props));
}

export function dispose2DMaterial(worldId: World2DId, materialId: MaterialId): void {
  disposeMaterialRaw(asWorldNumber(worldId), materialId as number);
}

export function dispose2DGeometry(worldId: World2DId, geometryId: GeometryId): void {
  disposeGeometryRaw(asWorldNumber(worldId), geometryId as number);
}

export function dispose2DTexture(worldId: World2DId, textureId: TextureId): void {
  disposeTextureRaw(asWorldNumber(worldId), textureId as number);
}

export function create2DSprite(
  worldId: World2DId,
  entityId: EntityId,
  props: { geometryId: GeometryId; materialId?: MaterialId; layer?: number }
): void {
  createSprite2DRaw(asWorldNumber(worldId), entityId as number, {
    geometryId: props.geometryId as number,
    materialId: props.materialId as number | undefined,
    layer: props.layer
  });
}

export function create2DShape(
  worldId: World2DId,
  entityId: EntityId,
  props: { geometryId: GeometryId; materialId?: MaterialId; layer?: number }
): void {
  createShape2DRaw(asWorldNumber(worldId), entityId as number, {
    geometryId: props.geometryId as number,
    materialId: props.materialId as number | undefined,
    layer: props.layer
  });
}

export function upsert2DRenderGraph(worldId: World2DId, args: CmdRenderGraphUpsertArgs): CommandId {
  return asCommandId(upsertRenderGraphRaw(asWorldNumber(worldId), args));
}

export function bind2DRenderGraph(worldId: World2DId, args: CmdRealmRenderGraphBindArgs): CommandId {
  return asCommandId(bindRealmRenderGraphRaw(asWorldNumber(worldId), args));
}

export function dispose2DRenderGraph(
  worldId: World2DId,
  args: CmdRenderGraphDisposeArgs
): CommandId {
  return asCommandId(disposeRenderGraphRaw(asWorldNumber(worldId), args));
}

export function list2DRenderGraphs(worldId: World2DId, args: CmdRenderGraphListArgs = {}): CommandId {
  return asCommandId(listRenderGraphsRaw(asWorldNumber(worldId), args));
}

export function upsert2DMaterialDefinition(
  worldId: World2DId,
  args: CmdMaterialDefinitionUpsertArgs
): CommandId {
  return asCommandId(upsertMaterialDefinitionRaw(asWorldNumber(worldId), args));
}

export function dispose2DMaterialDefinition(
  worldId: World2DId,
  args: CmdMaterialDefinitionDisposeArgs
): CommandId {
  return asCommandId(disposeMaterialDefinitionRaw(asWorldNumber(worldId), args));
}

export function upsert2DMaterialInstance(
  worldId: World2DId,
  args: CmdMaterialInstanceUpsertArgs
): CommandId {
  return asCommandId(upsertMaterialInstanceRaw(asWorldNumber(worldId), args));
}

export function dispose2DMaterialInstance(
  worldId: World2DId,
  args: CmdMaterialInstanceDisposeArgs
): CommandId {
  return asCommandId(disposeMaterialInstanceRaw(asWorldNumber(worldId), args));
}

export function list2DMaterials(worldId: World2DId): CommandId {
  return asCommandId(listMaterialsRaw(asWorldNumber(worldId), 'two-d'));
}

export function list2DTextures(worldId: World2DId): CommandId {
  return asCommandId(listTexturesRaw(asWorldNumber(worldId)));
}

export function list2DGeometries(worldId: World2DId): CommandId {
  return asCommandId(listGeometriesRaw(asWorldNumber(worldId)));
}

export function list2DCameras(worldId: World2DId): CommandId {
  return asCommandId(listCamerasRaw(asWorldNumber(worldId)));
}

export function measure2DTarget(worldId: World2DId, args: CmdTargetMeasurementArgs): CommandId {
  return asCommandId(measureTargetRaw(asWorldNumber(worldId), args));
}

export function send2DNotification(
  worldId: World2DId,
  args: { level: NotificationLevel; title: string; message: string }
): void {
  sendNotificationRaw(asWorldNumber(worldId), args);
}

export function get2DWorldRealmId(worldId: World2DId): number | null {
  return getWorldRealmIdRaw(asWorldNumber(worldId));
}
