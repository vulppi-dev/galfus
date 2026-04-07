import { createWorld3D as createWorld3DRaw } from '../api';
import type {
  CameraProps,
  GeometryProps,
  LightProps,
  MaterialProps,
  ModelProps,
  TagProps,
  TextureProps,
  TransformProps
} from '../ecs';
import type {
  CmdAudioListenerUpdateArgs,
  CmdAudioResourceUpsertArgs,
  CmdAudioSourceCreateArgs,
  CmdAudioSourceTransportArgs
} from '../../types/cmds/audio';
import type { EnvironmentConfig } from '../../types/cmds/environment';
import type {
  CmdGizmoDrawAabbArgs,
  CmdGizmoDrawLineArgs,
  CmdGizmoDrawPolylineArgs
} from '../../types/cmds/gizmo';
import type { CmdPoseUpdateArgs } from '../../types/cmds/model';
import type {
  CmdRealmRenderGraphBindArgs,
  CmdRenderGraphDisposeArgs,
  CmdRenderGraphListArgs,
  CmdRenderGraphUpsertArgs
} from '../../types/cmds/render-graph';
import type { ShadowConfig } from '../../types/cmds/shadow';
import type { CmdTargetMeasurementArgs } from '../../types/cmds/target';
import type { NotificationLevel } from '../../types/kinds';
import {
  audioListenerUpdate as audioListenerUpdateRaw,
  audioResourceCreate as audioResourceCreateRaw,
  audioSourceCreate as audioSourceCreateRaw,
  audioSourcePlay as audioSourcePlayRaw,
  configureEnvironment as configureEnvironmentRaw,
  configureShadows as configureShadowsRaw,
  createCamera as createCameraRaw,
  createEntity as createEntityRaw,
  createGeometry as createGeometryRaw,
  createLight as createLightRaw,
  createMaterial as createMaterialRaw,
  createModel as createModelRaw,
  createTexture as createTextureRaw,
  createTag as createTagRaw,
  disposeRenderGraph as disposeRenderGraphRaw,
  disposeGeometry as disposeGeometryRaw,
  disposeMaterial as disposeMaterialRaw,
  disposeTexture as disposeTextureRaw,
  drawGizmoAabb as drawGizmoAabbRaw,
  drawGizmoLine as drawGizmoLineRaw,
  drawGizmoPolyline as drawGizmoPolylineRaw,
  getModelId as getModelIdRaw,
  getWorldRealmId as getWorldRealmIdRaw,
  listCameras as listCamerasRaw,
  listGeometries as listGeometriesRaw,
  listLights as listLightsRaw,
  listMaterials as listMaterialsRaw,
  listModels as listModelsRaw,
  listRenderGraphs as listRenderGraphsRaw,
  listTextures as listTexturesRaw,
  measureTarget as measureTargetRaw,
  poseUpdate as poseUpdateRaw,
  removeEntity as removeEntityRaw,
  sendNotification as sendNotificationRaw,
  setParent as setParentRaw,
  bindRealmRenderGraph as bindRealmRenderGraphRaw,
  upsertRenderGraph as upsertRenderGraphRaw,
  updateTransform as updateTransformRaw
} from './entities';
import type { CommandId, EntityId, GeometryId, MaterialId, TextureId, World3DId } from './types';
import {
  asCommandId,
  asEntityId,
  asGeometryId,
  asMaterialId,
  asTextureId,
  asWorld3DId,
  asWorldNumber
} from './types';
export * from './world3d-input';

export type Create3DWorldOptions = {
  importance?: number;
  cachePolicy?: number;
  flags?: number;
};

type Create3DAudioSourceArgs = Omit<CmdAudioSourceCreateArgs, 'realmId' | 'modelId'> &
  ({ modelId: number } | { entityId: EntityId });

/**
 * Creates a 3D world.
 *
 * The world is realm-backed internally, but realm details are hidden from this API.
 * Use `Mount.mountWorld(...)` to present this world into one or more targets.
 */
export function create3DWorld(options?: Create3DWorldOptions): World3DId {
  return asWorld3DId(createWorld3DRaw(options));
}

/** Creates an entity in a 3D world. */
export function create3DEntity(worldId: World3DId): EntityId {
  return asEntityId(createEntityRaw(asWorldNumber(worldId)));
}

/** Removes an entity and all mirrored components from a 3D world. */
export function remove3DEntity(worldId: World3DId, entityId: EntityId): void {
  removeEntityRaw(asWorldNumber(worldId), entityId as number);
}

/** Upserts camera component intent for an entity in a 3D world. */
export function create3DCamera(worldId: World3DId, entityId: EntityId, props: CameraProps): void {
  createCameraRaw(asWorldNumber(worldId), entityId as number, props);
}

/** Upserts light component intent for an entity in a 3D world. */
export function create3DLight(worldId: World3DId, entityId: EntityId, props: LightProps): void {
  createLightRaw(asWorldNumber(worldId), entityId as number, props);
}

/** Upserts model component intent for an entity in a 3D world. */
export function create3DModel(worldId: World3DId, entityId: EntityId, props: ModelProps): void {
  createModelRaw(asWorldNumber(worldId), entityId as number, props);
}

/** Upserts transform component intent for an entity in a 3D world. */
export function update3DTransform(
  worldId: World3DId,
  entityId: EntityId,
  props: TransformProps
): void {
  updateTransformRaw(asWorldNumber(worldId), entityId as number, props);
}

/** Attaches or updates a tag component in a 3D world. */
export function create3DTag(worldId: World3DId, entityId: EntityId, props: TagProps): void {
  createTagRaw(asWorldNumber(worldId), entityId as number, props);
}

/** Sets parent-child relationship between entities. */
export function set3DParent(
  worldId: World3DId,
  childEntityId: EntityId,
  parentEntityId: EntityId | null
): void {
  setParentRaw(asWorldNumber(worldId), childEntityId as number, parentEntityId as number | null);
}

/** Creates a material resource and returns its typed id. */
export function create3DMaterial(worldId: World3DId, props: MaterialProps): MaterialId {
  return asMaterialId(createMaterialRaw(asWorldNumber(worldId), props));
}

/** Creates a geometry resource and returns its typed id. */
export function create3DGeometry(worldId: World3DId, props: GeometryProps): GeometryId {
  return asGeometryId(createGeometryRaw(asWorldNumber(worldId), props));
}

/** Creates a texture resource and returns its typed id. */
export function create3DTexture(worldId: World3DId, props: TextureProps): TextureId {
  return asTextureId(createTextureRaw(asWorldNumber(worldId), props));
}

/** Disposes a material resource from a 3D world. */
export function dispose3DMaterial(worldId: World3DId, materialId: MaterialId): void {
  disposeMaterialRaw(asWorldNumber(worldId), materialId as number);
}

/** Disposes a geometry resource from a 3D world. */
export function dispose3DGeometry(worldId: World3DId, geometryId: GeometryId): void {
  disposeGeometryRaw(asWorldNumber(worldId), geometryId as number);
}

/** Disposes a texture resource from a 3D world. */
export function dispose3DTexture(worldId: World3DId, textureId: TextureId): void {
  disposeTextureRaw(asWorldNumber(worldId), textureId as number);
}

/** Configures environment/post-processing for a 3D world. */
export function configure3DEnvironment(worldId: World3DId, config: EnvironmentConfig): void {
  configureEnvironmentRaw(asWorldNumber(worldId), config);
}

/** Configures shadows for a 3D world. */
export function configure3DShadows(worldId: World3DId, config: ShadowConfig): void {
  configureShadowsRaw(asWorldNumber(worldId), config);
}

/** Draws a debug line gizmo. */
export function draw3DGizmoLine(worldId: World3DId, args: CmdGizmoDrawLineArgs): void {
  drawGizmoLineRaw(asWorldNumber(worldId), args);
}

/** Draws a debug axis-aligned bounding box gizmo. */
export function draw3DGizmoAabb(worldId: World3DId, args: CmdGizmoDrawAabbArgs): void {
  drawGizmoAabbRaw(asWorldNumber(worldId), args);
}

/** Draws a debug polyline gizmo. */
export function draw3DGizmoPolyline(worldId: World3DId, args: CmdGizmoDrawPolylineArgs): void {
  drawGizmoPolylineRaw(asWorldNumber(worldId), args);
}

/** Updates pose data for XR/trackers. */
export function update3DPose(worldId: World3DId, args: CmdPoseUpdateArgs): CommandId {
  return asCommandId(poseUpdateRaw(asWorldNumber(worldId), args));
}

/** Requests a model list from core for this world. */
export function list3DModels(worldId: World3DId): CommandId {
  return asCommandId(listModelsRaw(asWorldNumber(worldId)));
}

/** Requests a material list from core for this world. */
export function list3DMaterials(worldId: World3DId): CommandId {
  return asCommandId(listMaterialsRaw(asWorldNumber(worldId)));
}

/** Requests a texture list from core for this world. */
export function list3DTextures(worldId: World3DId): CommandId {
  return asCommandId(listTexturesRaw(asWorldNumber(worldId)));
}

/** Requests a geometry list from core for this world. */
export function list3DGeometries(worldId: World3DId): CommandId {
  return asCommandId(listGeometriesRaw(asWorldNumber(worldId)));
}

/** Requests a light list from core for this world. */
export function list3DLights(worldId: World3DId): CommandId {
  return asCommandId(listLightsRaw(asWorldNumber(worldId)));
}

/** Requests a camera list from core for this world. */
export function list3DCameras(worldId: World3DId): CommandId {
  return asCommandId(listCamerasRaw(asWorldNumber(worldId)));
}

/** Creates or updates a render graph definition in core. */
export function upsert3DRenderGraph(worldId: World3DId, args: CmdRenderGraphUpsertArgs): CommandId {
  return asCommandId(upsertRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Disposes a render graph definition from core. */
export function dispose3DRenderGraph(
  worldId: World3DId,
  args: CmdRenderGraphDisposeArgs
): CommandId {
  return asCommandId(disposeRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Requests render graph catalog from core. */
export function list3DRenderGraphs(
  worldId: World3DId,
  args: CmdRenderGraphListArgs = {}
): CommandId {
  return asCommandId(listRenderGraphsRaw(asWorldNumber(worldId), args));
}

/** Binds realm to a render graph id. */
export function bind3DRealmRenderGraph(
  worldId: World3DId,
  args: CmdRealmRenderGraphBindArgs
): CommandId {
  return asCommandId(bindRealmRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Requests target measurement for this world context. */
export function measure3DTarget(worldId: World3DId, args: CmdTargetMeasurementArgs): CommandId {
  return asCommandId(measureTargetRaw(asWorldNumber(worldId), args));
}

/** Sends a host notification scoped to this world. */
export function send3DNotification(
  worldId: World3DId,
  args: { level: NotificationLevel; title: string; message: string }
): void {
  sendNotificationRaw(asWorldNumber(worldId), args);
}

/** Updates audio listener parameters. */
export function update3DAudioListener(
  worldId: World3DId,
  args: CmdAudioListenerUpdateArgs
): CommandId {
  return asCommandId(audioListenerUpdateRaw(asWorldNumber(worldId), args));
}

/** Creates or updates an audio resource. */
export function create3DAudioResource(
  worldId: World3DId,
  args: CmdAudioResourceUpsertArgs
): CommandId {
  return asCommandId(audioResourceCreateRaw(asWorldNumber(worldId), args));
}

/** Creates an audio source with realm id resolved internally from world state. */
export function create3DAudioSource(worldId: World3DId, args: Create3DAudioSourceArgs): CommandId {
  const rawWorldId = asWorldNumber(worldId);
  const realmId = getWorldRealmIdRaw(rawWorldId) ?? rawWorldId;

  const modelId =
    'modelId' in args
      ? args.modelId
      : (getModelIdRaw(rawWorldId, args.entityId as number) ?? (args.entityId as number));
  const baseArgs = 'modelId' in args ? args : (({ entityId: _entityId, ...rest }) => rest)(args);

  return asCommandId(
    audioSourceCreateRaw(rawWorldId, {
      ...baseArgs,
      modelId,
      realmId
    })
  );
}

/** Starts playback on an audio source. */
export function play3DAudioSource(
  worldId: World3DId,
  args: Omit<CmdAudioSourceTransportArgs, 'action'>
): CommandId {
  return asCommandId(audioSourcePlayRaw(asWorldNumber(worldId), args));
}
