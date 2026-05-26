import type { Vec4 as vec4 } from '../../math/index';
import type {
  PrimitiveTopology,
  PolygonMode,
  RenderSide,
  SamplerMode,
  TransparencyMode
} from '../kinds';
import type {
  CmdResourceGetArgs,
  CmdResourceListArgs,
  CmdResultResourceGet,
  CmdResultResourceList,
  ResourceEntry
} from './resources';

/** Material options payload accepted by core (schema-only). */
export type MaterialOptions = {
  type: 'schema';
  content: Record<string, vec4>;
};

/** Command payload for creating a material. */
export type MaterialRealmKind = 'two-d' | 'three-d';

export interface CmdMaterialCreateArgs {
  materialId: number;
  label?: string;
  slug: string;
  kind: 'shader';
  realmKind?: MaterialRealmKind;
  options?: MaterialOptions;
}

/** Command payload for updating a material. */
export interface CmdMaterialUpdateArgs {
  materialId: number;
  label?: string;
  slug?: string;
  kind?: 'shader';
  realmKind?: MaterialRealmKind;
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
  realmKind?: MaterialRealmKind;
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
  realmKind: MaterialRealmKind;
  preset?: string;
  shaderType?: string;
  shaderSource?: string;
  shaderParamsSchema?: Record<string, string>;
  capabilities?: MaterialShaderCapabilities;
}

export interface CmdMaterialDefinitionUpdateArgs {
  definitionId: number;
  slug?: string;
  label?: string;
  realmKind?: MaterialRealmKind;
  preset?: string;
  shaderType?: string;
  shaderSource?: string;
  shaderParamsSchema?: Record<string, string>;
  capabilities?: MaterialShaderCapabilities;
}

export interface MaterialShaderCapabilities {
  semantics?: string[];
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

export type CmdMaterialInstanceUpsertArgs =
  | CmdMaterialInstanceCreateArgs
  | CmdMaterialInstanceUpdateArgs;

export interface CmdMaterialInstanceDisposeArgs {
  materialId: number;
}

export interface CmdResultMaterialInstance {
  success: boolean;
  message: string;
}

export interface CmdMaterialGetArgs extends CmdResourceGetArgs {
  realmKind?: MaterialRealmKind;
}
export type CmdResultMaterialGet = CmdResultResourceGet;
export type CmdMaterialDefinitionGetArgs = CmdResourceGetArgs;
export type CmdResultMaterialDefinitionGet = CmdResultResourceGet;
export type CmdMaterialDefinitionListArgs = CmdResourceListArgs;
export type CmdResultMaterialDefinitionList = CmdResultResourceList;
export interface CmdMaterialInstanceGetArgs extends CmdResourceGetArgs {
  realmKind?: MaterialRealmKind;
}
export type CmdResultMaterialInstanceGet = CmdResultResourceGet;
export interface CmdMaterialInstanceListArgs extends CmdResourceListArgs {
  realmKind?: MaterialRealmKind;
}
export type CmdResultMaterialInstanceList = CmdResultResourceList;
