import type { Vec3 as vec3 } from '../../math/index';
import type { PrimitiveShape } from '../kinds';
import type { ResourceEntry } from './resources';

/** Primitive attribute semantic identifiers. */
export type GeometryPrimitiveType =
  | 'index'
  | 'position'
  | 'normal'
  | 'tangent'
  | 'color'
  | 'u-v'
  | 'skin-joints'
  | 'skin-weights';

/** Convenience mapping for primitive attribute identifiers. */
export const GeometryPrimitiveType = {
  Index: 'index' as const,
  Position: 'position' as const,
  Normal: 'normal' as const,
  Tangent: 'tangent' as const,
  Color: 'color' as const,
  UV: 'u-v' as const,
  SkinJoints: 'skin-joints' as const,
  SkinWeights: 'skin-weights' as const
};

/** Geometry buffer entry describing a single attribute stream. */
export interface GeometryPrimitiveEntry {
  primitiveType: GeometryPrimitiveType;
  bufferId: number;
}

/** Command payload for creating geometry from buffers. */
export interface CmdGeometryCreateArgs {
  geometryId: number;
  label?: string;
  entries: GeometryPrimitiveEntry[];
}

/** Command payload for updating geometry buffers or label. */
export interface CmdGeometryUpdateArgs {
  geometryId: number;
  label?: string;
  entries?: GeometryPrimitiveEntry[];
}

/** Result payload for geometry upsert. */
export interface CmdResultGeometryUpsert {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdGeometryUpsertArgs = CmdGeometryCreateArgs | CmdGeometryUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultGeometryCreate = CmdResultGeometryUpsert;
export type CmdResultGeometryUpdate = CmdResultGeometryUpsert;

/** Command payload for disposing geometry. */
export interface CmdGeometryDisposeArgs {
  geometryId: number;
}

/** Result payload for geometry dispose. */
export interface CmdResultGeometryDispose {
  success: boolean;
  message: string;
}

/** Options for cube primitive generation. */
export interface CubeOptions {
  size: vec3;
  subdivisions: number;
}

/** Options for plane primitive generation. */
export interface PlaneOptions {
  size: vec3;
  subdivisions: number;
}

/** Options for sphere primitive generation. */
export interface SphereOptions {
  radius: number;
  sectors: number;
  stacks: number;
}

/** Options for cylinder primitive generation. */
export interface CylinderOptions {
  radius: number;
  height: number;
  sectors: number;
}

/** Options for torus primitive generation. */
export interface TorusOptions {
  majorRadius: number;
  minorRadius: number;
  majorSegments: number;
  minorSegments: number;
}

/** Options for pyramid primitive generation. */
export interface PyramidOptions {
  size: vec3;
  subdivisions: number;
}

/** Options for pill primitive generation. */
export interface PillOptions {
  radius: number;
  height: number;
  sectors: number;
  stacks: number;
}

/** Union of all primitive geometry options. */
export type PrimitiveOptions =
  | { type: 'cube'; content: CubeOptions }
  | { type: 'plane'; content: PlaneOptions }
  | { type: 'sphere'; content: SphereOptions }
  | { type: 'cylinder'; content: CylinderOptions }
  | { type: 'torus'; content: TorusOptions }
  | { type: 'pyramid'; content: PyramidOptions }
  | { type: 'pill'; content: PillOptions };

/** Command payload for creating a primitive geometry. */
export interface CmdPrimitiveGeometryCreateArgs {
  geometryId: number;
  label?: string;
  shape: PrimitiveShape;
  options?: PrimitiveOptions;
}

/** Result payload for primitive geometry create. */
export interface CmdResultPrimitiveGeometryCreate {
  success: boolean;
  message: string;
}

/** Command payload for listing geometry resources. */
export interface CmdGeometryListArgs {
  windowId: number;
}

/** Result payload for geometry list. */
export interface CmdResultGeometryList {
  success: boolean;
  message: string;
  geometries: ResourceEntry[];
}
