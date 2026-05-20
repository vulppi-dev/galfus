import type { Vec3 as vec3, Vec4 as vec4 } from '../../math/index';
import type { SkyboxMode } from '../kinds';
import type {
  CmdResourceGetArgs,
  CmdResourceListArgs,
  CmdResultResourceGet,
  CmdResultResourceList
} from './resources';

/** MSAA configuration for the renderer. */
export interface MsaaConfig {
  enabled?: boolean;
  sampleCount?: number;
}

/** Skybox configuration for the environment. */
export interface SkyboxConfig {
  mode?: SkyboxMode;
  intensity?: number;
  rotation?: number;
  groundColor?: vec3;
  horizonColor?: vec3;
  skyColor?: vec3;
  horizonGroundThreshold?: number;
  horizonSkyThreshold?: number;
  directionalLights?: SkyboxDirectionalLightSun[];
  cubemapTextureId?: number | null;
}

export interface SkyboxDirectionalLightSun {
  lightId: number;
  solidSize?: number;
  gradientSize?: number;
}

/** Post-processing configuration. */
export interface PostProcessConfig {
  filterEnabled?: boolean;
  filterExposure?: number;
  filterGamma?: number;
  filterSaturation?: number;
  filterContrast?: number;
  filterVignette?: number;
  filterGrain?: number;
  filterChromaticAberration?: number;
  filterBlur?: number;
  filterSharpen?: number;
  filterTonemapMode?: number;
  outlineEnabled?: boolean;
  outlineStrength?: number;
  outlineThreshold?: number;
  outlineWidth?: number;
  outlineQuality?: number;
  filterPosterizeSteps?: number;
  cellShading?: boolean;
  ssaoEnabled?: boolean;
  ssaoStrength?: number;
  ssaoRadius?: number;
  ssaoBias?: number;
  ssaoPower?: number;
  ssaoBlurRadius?: number;
  ssaoBlurDepthThreshold?: number;
  bloomEnabled?: boolean;
  bloomThreshold?: number;
  bloomKnee?: number;
  bloomIntensity?: number;
  bloomScatter?: number;
}

/** Environment configuration for a window. */
export interface EnvironmentConfig {
  msaa?: MsaaConfig;
  skybox?: SkyboxConfig;
  clearColor?: vec4;
  post?: PostProcessConfig;
}

/** Command payload for creating environment settings. */
export interface CmdEnvironmentCreateArgs {
  environmentId: number;
  config: EnvironmentConfig;
}

/** Command payload for updating environment settings. */
export interface CmdEnvironmentUpdateArgs {
  environmentId: number;
  config: EnvironmentConfig;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdEnvironmentUpsertArgs = CmdEnvironmentCreateArgs | CmdEnvironmentUpdateArgs;

/** Command payload for disposing environment settings. */
export interface CmdEnvironmentDisposeArgs {
  environmentId: number;
}

/** Result payload for environment commands. */
export interface CmdResultEnvironment {
  success: boolean;
  message: string;
}

export type CmdEnvironmentGetArgs = CmdResourceGetArgs;
export type CmdResultEnvironmentGet = CmdResultResourceGet;
export type CmdEnvironmentListArgs = CmdResourceListArgs;
export type CmdResultEnvironmentList = CmdResultResourceList;
