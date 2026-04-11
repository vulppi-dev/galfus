import type { Vec2 as vec2, Vec4 as vec4 } from '../../math/index';
import type { LightKind } from '../kinds';
import type { ResourceEntry } from './resources';

/** Command payload for creating a light. */
export interface CmdLightCreateArgs {
  realmId: number;
  lightId: number;
  label?: string;
  kind?: LightKind;
  position?: vec4;
  direction?: vec4;
  color?: vec4;
  groundColor?: vec4;
  intensity?: number;
  range?: number;
  spotInnerOuter?: vec2;
  layerMask?: number;
  castShadow?: boolean;
}

/** Command payload for updating a light. */
export interface CmdLightUpdateArgs {
  realmId: number;
  lightId: number;
  label?: string;
  kind?: LightKind;
  position?: vec4;
  direction?: vec4;
  color?: vec4;
  groundColor?: vec4;
  intensity?: number;
  range?: number;
  spotInnerOuter?: vec2;
  layerMask?: number;
  castShadow?: boolean;
}

/** Result payload for light upsert. */
export interface CmdResultLightUpsert {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdLightUpsertArgs = CmdLightCreateArgs | CmdLightUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultLightCreate = CmdResultLightUpsert;
export type CmdResultLightUpdate = CmdResultLightUpsert;

/** Command payload for disposing a light. */
export interface CmdLightDisposeArgs {
  realmId: number;
  lightId: number;
}

/** Result payload for light dispose. */
export interface CmdResultLightDispose {
  success: boolean;
  message: string;
}

/** Command payload for listing lights. */
export interface CmdLightListArgs {
  windowId: number;
}

/** Result payload for light list. */
export interface CmdResultLightList {
  success: boolean;
  message: string;
  lights: ResourceEntry[];
}
