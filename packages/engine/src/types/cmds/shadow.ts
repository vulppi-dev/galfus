/** Shadow rendering configuration. */
export interface ShadowConfig {
  tileResolution?: number;
  atlasTilesW?: number;
  atlasTilesH?: number;
  atlasLayers?: number;
  virtualGridSize?: number;
  smoothing?: number;
  normalBias?: number;
}

/** Command payload for shadow configuration. */
export interface CmdShadowConfigureArgs {
  windowId: number;
  config: ShadowConfig;
}

/** Result payload for shadow configuration. */
export interface CmdResultShadowConfigure {
  success: boolean;
  message: string;
}
