import type { Vec4 as vec4 } from '../../math/index';
import type {
  MaterialKind,
  TransparencyMode,
  SamplerMode,
  PrimitiveTopology,
  PolygonMode,
  RenderSide
} from '../kinds';
import type { ResourceEntry } from './resources';

/** Standard material options. */
export interface StandardOptions {
  baseColor?: vec4;
  surfaceType?: TransparencyMode;
  topology?: PrimitiveTopology;
  polygonMode?: PolygonMode;
  renderSide?: RenderSide;
  emissiveColor?: vec4 | null;
  specColor?: vec4 | null;
  specPower?: number | null;
  baseTexId?: number | null;
  baseSampler?: SamplerMode | null;
  specTexId?: number | null;
  specSampler?: SamplerMode | null;
  normalTexId?: number | null;
  normalSampler?: SamplerMode | null;
  toonRampTexId?: number | null;
  toonRampSampler?: SamplerMode | null;
  emissiveTexId?: number | null;
  emissiveSampler?: SamplerMode | null;
  flags?: number;
  toonParams?: vec4 | null;
}

/** PBR material options. */
export interface PbrOptions {
  baseColor?: vec4;
  surfaceType?: TransparencyMode;
  topology?: PrimitiveTopology;
  polygonMode?: PolygonMode;
  renderSide?: RenderSide;
  emissiveColor?: vec4;
  metallic?: number;
  roughness?: number;
  ao?: number;
  normalScale?: number;
  baseTexId?: number | null;
  baseSampler?: SamplerMode | null;
  normalTexId?: number | null;
  normalSampler?: SamplerMode | null;
  metallicRoughnessTexId?: number | null;
  metallicRoughnessSampler?: SamplerMode | null;
  emissiveTexId?: number | null;
  emissiveSampler?: SamplerMode | null;
  aoTexId?: number | null;
  aoSampler?: SamplerMode | null;
  flags?: number;
}

/** Union of material option payloads. */
export type MaterialOptions =
  | { type: 'standard'; content: StandardOptions }
  | { type: 'pbr'; content: PbrOptions };

/** Command payload for creating a material. */
export interface CmdMaterialCreateArgs {
  materialId: number;
  label?: string;
  kind: MaterialKind;
  options?: MaterialOptions;
}

/** Command payload for updating a material. */
export interface CmdMaterialUpdateArgs {
  materialId: number;
  label?: string;
  kind?: MaterialKind;
  options?: MaterialOptions;
}

/** Result payload for material upsert. */
export interface CmdResultMaterialUpsert {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdMaterialUpsertArgs = CmdMaterialCreateArgs | CmdMaterialUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultMaterialCreate = CmdResultMaterialUpsert;
export type CmdResultMaterialUpdate = CmdResultMaterialUpsert;

/** Command payload for disposing a material. */
export interface CmdMaterialDisposeArgs {
  materialId: number;
}

/** Result payload for material dispose. */
export interface CmdResultMaterialDispose {
  success: boolean;
  message: string;
}

/** Command payload for listing materials. */
export interface CmdMaterialListArgs {
  windowId: number;
}

/** Result payload for material list. */
export interface CmdResultMaterialList {
  success: boolean;
  message: string;
  materials: ResourceEntry[];
}
