import type { Vec4 as vec4 } from '../../math/index';
import type { TextureCreateMode } from '../kinds';
import type { CmdResourceGetArgs, CmdResultResourceGet, ResourceEntry } from './resources';

/** Options for forward atlas texture packing. */
export interface ForwardAtlasOptions {
  tilePx?: number;
  layers?: number;
}

/** Command payload for creating a texture from an uploaded buffer. */
export interface CmdTextureCreateFromBufferArgs {
  textureId: number;
  label?: string;
  bufferId: number;
  srgb?: boolean;
  mode?: TextureCreateMode;
  atlasOptions?: ForwardAtlasOptions;
}

/** Result payload for texture create from buffer. */
export interface CmdResultTextureCreateFromBuffer {
  success: boolean;
  message: string;
  pending: boolean;
}

/** Command payload for creating a solid color texture. */
export interface CmdTextureCreateSolidColorArgs {
  textureId: number;
  label?: string;
  color: vec4;
  srgb?: boolean;
  mode?: TextureCreateMode;
  atlasOptions?: ForwardAtlasOptions;
}

/** Result payload for solid color texture create. */
export interface CmdResultTextureCreateSolidColor {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by core. */
export type CmdTextureUpsertArgs = CmdTextureCreateFromBufferArgs | CmdTextureCreateSolidColorArgs;

/** Result payload for texture upsert. */
export interface CmdResultTextureUpsert {
  success: boolean;
  message: string;
}

/** Legacy aliases kept for one cycle. */
export type CmdTextureCreateArgs = CmdTextureUpsertArgs;

/** Command payload for disposing a texture. */
export interface CmdTextureDisposeArgs {
  textureId: number;
}

/** Result payload for texture dispose. */
export interface CmdResultTextureDispose {
  success: boolean;
  message: string;
}

/** Command payload for binding a texture to a target output. */
export interface CmdTextureBindTargetArgs {
  textureId: number;
  targetId: number;
  label?: string;
}

/** Result payload for texture-target binding. */
export interface CmdResultTextureBindTarget {
  success: boolean;
  message: string;
}

/** Command payload for listing textures. */
export interface CmdTextureListArgs {
  windowId: number;
}

/** Result payload for texture list. */
export interface CmdResultTextureList {
  success: boolean;
  message: string;
  textures: ResourceEntry[];
}

export type CmdTextureGetArgs = CmdResourceGetArgs;
export type CmdResultTextureGet = CmdResultResourceGet;
