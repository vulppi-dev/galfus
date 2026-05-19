import type { Vec2 as vec2, Vec4 as vec4 } from '../../math/index';
export type TargetKind = 'window' | 'texture';

export type SurfaceFormat =
  | 'rgba16-float'
  | 'rgba8-unorm'
  | 'bgra8-unorm'
  | 'depth32-float'
  | 'depth24-plus';

export type SurfaceAlphaMode = 'auto' | 'opaque' | 'pre-multiplied' | 'post-multiplied' | 'inherit';

export type DimensionValue =
  | { unit: 'px'; value: number }
  | { unit: 'percent'; value: number }
  | { unit: 'character'; value: number }
  | { unit: 'display'; value: number };

export interface TargetLayerLayout {
  left: DimensionValue;
  top: DimensionValue;
  width: DimensionValue;
  height: DimensionValue;
  enabled?: boolean;
  opacity?: number;
  zIndex: number;
  blendMode: number;
  clip?: vec4 | null;
}

export interface CmdTargetUpsertArgs {
  targetId: number;
  kind: TargetKind;
  windowId?: number;
  size?: vec2;
  formatPolicy?: SurfaceFormat;
  alphaPolicy?: SurfaceAlphaMode;
  msaaSamples?: number;
}

export interface CmdResultTargetUpsert {
  success: boolean;
  message: string;
}

export interface CmdTargetMeasurementArgs {
  targetId: number;
  getSize?: boolean;
  getWindowSize?: boolean;
}

export interface CmdResultTargetMeasurement {
  success: boolean;
  message: string;
  size?: vec2;
  windowSize?: vec2;
  sourceKind?: string;
}

export interface CmdTargetDisposeArgs {
  targetId: number;
}

export interface CmdResultTargetDispose {
  success: boolean;
  message: string;
}

export interface CmdTargetLayerUpsertArgs {
  realmId: number;
  targetId: number;
  layout: TargetLayerLayout;
  cameraId?: number;
  environmentId?: number;
}

export interface CmdResultTargetLayerUpsert {
  success: boolean;
  message: string;
}

export interface CmdTargetLayerDisposeArgs {
  realmId: number;
  targetId: number;
}

export interface CmdResultTargetLayerDispose {
  success: boolean;
  message: string;
}
