import type { mat4, vec2 } from 'gl-matrix';
import type { CameraKind } from '../kinds';
import type { ResourceEntry } from './resources';

/** Viewport value expressed as relative (0..1) or absolute pixels. */
export interface ViewValue {
  type: 'relative' | 'absolute';
  value: number;
}

/** Viewport definition relative to the window. */
export interface ViewPosition {
  anchor: { x: ViewValue; y: ViewValue };
  size: { width: ViewValue; height: ViewValue };
}

/** Command payload for creating a camera. */
export interface CmdCameraCreateArgs {
  realmId: number;
  cameraId: number;
  label?: string;
  transform: mat4;
  kind: CameraKind;
  flags?: number;
  nearFar: vec2;
  layerMask?: number;
  order?: number;
  viewPosition?: ViewPosition;
  orthoScale?: number;
}

/** Command payload for updating a camera. */
export interface CmdCameraUpdateArgs {
  realmId: number;
  cameraId: number;
  label?: string;
  transform?: mat4;
  kind?: CameraKind;
  flags?: number;
  layerMask?: number;
  order?: number;
  nearFar?: vec2;
  viewPosition?: ViewPosition;
  orthoScale?: number;
}

/** Result payload for camera upsert. */
export interface CmdResultCameraUpsert {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdCameraUpsertArgs = CmdCameraCreateArgs | CmdCameraUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultCameraCreate = CmdResultCameraUpsert;
export type CmdResultCameraUpdate = CmdResultCameraUpsert;

/** Command payload for disposing a camera. */
export interface CmdCameraDisposeArgs {
  realmId: number;
  cameraId: number;
}

/** Result payload for camera dispose. */
export interface CmdResultCameraDispose {
  success: boolean;
  message: string;
}

/** Command payload for listing cameras. */
export interface CmdCameraListArgs {
  windowId: number;
}

/** Result payload for camera list. */
export interface CmdResultCameraList {
  success: boolean;
  message: string;
  cameras: ResourceEntry[];
}
