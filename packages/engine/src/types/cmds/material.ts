import type { Vec4 as vec4 } from '../../math/index';
import type { TransparencyMode, SamplerMode, PrimitiveTopology, PolygonMode, RenderSide } from '../kinds';
import type {
  CmdResourceGetArgs,
  CmdResourceListArgs,
  CmdResultResourceGet,
  CmdResultResourceList,
  ResourceEntry
} from './resources';

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
  slug: string;
  kind: 'shader';
  options?: MaterialOptions;
}

/** Command payload for updating a material. */
export interface CmdMaterialUpdateArgs {
  materialId: number;
  label?: string;
  slug?: string;
  kind?: 'shader';
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

export interface CmdMaterialDefinitionCreateArgs {
  definitionId: number;
  slug: string;
  label?: string;
  preset: string;
  shaderType?: string;
  shaderSource: string;
  shaderParamsSchema?: Record<string, string>;
}

export interface CmdMaterialDefinitionUpdateArgs {
  definitionId: number;
  slug?: string;
  label?: string;
  preset?: string;
  shaderType?: string;
  shaderSource: string;
  shaderParamsSchema?: Record<string, string>;
}

export type CmdMaterialDefinitionUpsertArgs =
  | CmdMaterialDefinitionCreateArgs
  | CmdMaterialDefinitionUpdateArgs;

export interface CmdMaterialDefinitionDisposeArgs {
  definitionId: number;
}

export interface CmdResultMaterialDefinition {
  success: boolean;
  message: string;
}

export interface CmdMaterialInstanceCreateArgs {
  materialId: number;
  slug: string;
  label?: string;
  options?: MaterialOptions;
}

export interface CmdMaterialInstanceUpdateArgs {
  materialId: number;
  slug?: string;
  label?: string;
  options?: MaterialOptions;
}

export type CmdMaterialInstanceUpsertArgs = CmdMaterialInstanceCreateArgs | CmdMaterialInstanceUpdateArgs;

export interface CmdMaterialInstanceDisposeArgs {
  materialId: number;
}

export interface CmdResultMaterialInstance {
  success: boolean;
  message: string;
}

export type CmdMaterialGetArgs = CmdResourceGetArgs;
export type CmdResultMaterialGet = CmdResultResourceGet;
export type CmdMaterialDefinitionGetArgs = CmdResourceGetArgs;
export type CmdResultMaterialDefinitionGet = CmdResultResourceGet;
export type CmdMaterialDefinitionListArgs = CmdResourceListArgs;
export type CmdResultMaterialDefinitionList = CmdResultResourceList;
export type CmdMaterialInstanceGetArgs = CmdResourceGetArgs;
export type CmdResultMaterialInstanceGet = CmdResultResourceGet;
export type CmdMaterialInstanceListArgs = CmdResourceListArgs;
export type CmdResultMaterialInstanceList = CmdResultResourceList;
