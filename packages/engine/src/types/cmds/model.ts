import type { ResourceEntry } from './resources';

/** Command payload for creating a model. */
export interface CmdModelCreateArgs {
  realmId: number;
  modelId: number;
  label?: string;
  geometryId: number;
  materialId?: number;
  transform: number[]; // Mat4
  layerMask?: number;
  castShadow?: boolean;
  receiveShadow?: boolean;
  castOutline?: boolean;
  outlineColor?: [number, number, number, number];
}

/** Command payload for updating a model. */
export interface CmdModelUpdateArgs {
  realmId: number;
  modelId: number;
  label?: string;
  geometryId?: number;
  materialId?: number;
  transform?: number[];
  layerMask?: number;
  castShadow?: boolean;
  receiveShadow?: boolean;
  castOutline?: boolean;
  outlineColor?: [number, number, number, number];
}

/** Result payload for model upsert. */
export interface CmdResultModelUpsert {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdModelUpsertArgs = CmdModelCreateArgs | CmdModelUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultModelCreate = CmdResultModelUpsert;
export type CmdResultModelUpdate = CmdResultModelUpsert;

/** Command payload for updating a model pose (skinning). */
export interface CmdPoseUpdateArgs {
  realmId: number;
  modelId: number;
  boneCount: number;
  matricesBufferId: number;
  windowId?: number;
}

/** Result payload for pose update. */
export interface CmdResultPoseUpdate {
  success: boolean;
  message: string;
}

/** Command payload for disposing a model. */
export interface CmdModelDisposeArgs {
  realmId: number;
  modelId: number;
}

/** Result payload for model dispose. */
export interface CmdResultModelDispose {
  success: boolean;
  message: string;
}

/** Command payload for listing models. */
export interface CmdModelListArgs {
  windowId: number;
}

/** Result payload for model list. */
export interface CmdResultModelList {
  success: boolean;
  message: string;
  models: ResourceEntry[];
}
