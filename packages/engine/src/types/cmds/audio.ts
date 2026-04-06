import type { AudioPlayMode } from '../kinds';

export interface AudioSpatialParams {
  minDistance?: number;
  maxDistance?: number;
  rolloff?: number;
  coneInner?: number;
  coneOuter?: number;
  coneOuterGain?: number;
}

/** Command payload for updating the listener transform explicitly. */
export interface CmdAudioListenerUpdateArgs {
  position: [number, number, number];
  velocity: [number, number, number];
  forward: [number, number, number];
  up: [number, number, number];
}

/** Command payload for binding the listener to a model. */
export interface CmdAudioListenerCreateArgs {
  realmId: number;
  modelId: number;
}

/** Command payload for disposing the listener binding. */
export interface CmdAudioListenerDisposeArgs {
  realmId: number;
}

export interface CmdResultAudioListenerCreate {
  success: boolean;
  message: string;
}

export interface CmdResultAudioListenerDispose {
  success: boolean;
  message: string;
}

export interface CmdResultAudioListenerUpdate {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdAudioListenerUpsertArgs =
  | CmdAudioListenerCreateArgs
  | CmdAudioListenerUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultAudioListenerUpsert = CmdResultAudioListenerUpdate;

/** Command payload for creating/updating an audio resource from uploaded bytes. */
export interface CmdAudioResourceUpsertArgs {
  resourceId: number;
  bufferId: number;
  totalBytes?: number;
  offsetBytes?: number;
}

export interface CmdResultAudioResourceUpsert {
  success: boolean;
  message: string;
  pending: boolean;
  receivedBytes: number;
  totalBytes: number;
  complete: boolean;
}

/** Command payload for creating a source bound to a model. */
export interface CmdAudioSourceCreateArgs {
  realmId: number;
  sourceId: number;
  modelId: number;
  position?: [number, number, number];
  velocity?: [number, number, number];
  orientation?: [number, number, number, number];
  gain?: number;
  pitch?: number;
  spatial?: AudioSpatialParams;
}

export interface CmdResultAudioSourceCreate {
  success: boolean;
  message: string;
}

/** Command payload for updating source params. */
export interface CmdAudioSourceUpdateArgs {
  sourceId: number;
  realmId?: number;
  modelId?: number;
  position?: [number, number, number];
  velocity?: [number, number, number];
  orientation?: [number, number, number, number];
  gain?: number;
  pitch?: number;
  spatial?: AudioSpatialParams;
}

export interface CmdResultAudioSourceUpdate {
  success: boolean;
  message: string;
}

/** Upsert payload accepted by the core (`create` or `update`). */
export type CmdAudioSourceUpsertArgs =
  | CmdAudioSourceCreateArgs
  | CmdAudioSourceUpdateArgs;

/** Backward-compatible aliases. */
export type CmdResultAudioSourceUpsert = CmdResultAudioSourceUpdate;

export type AudioSourceTransportAction = 'play' | 'pause' | 'stop';

/** Command payload for playback transport control. */
export interface CmdAudioSourceTransportArgs {
  sourceId: number;
  action: AudioSourceTransportAction;
  resourceId?: number;
  timelineId?: number;
  intensity?: number;
  delayMs?: number;
  mode?: AudioPlayMode;
}

export interface CmdResultAudioSourceTransport {
  success: boolean;
  message: string;
}

/** Command payload for disposing a source. */
export interface CmdAudioSourceDisposeArgs {
  sourceId: number;
}

export interface CmdResultAudioSourceDispose {
  success: boolean;
  message: string;
}

/** Command payload for disposing an audio resource. */
export interface CmdAudioResourceDisposeArgs {
  resourceId: number;
}

export interface CmdResultAudioResourceDispose {
  success: boolean;
  message: string;
}

/** Command payload for listing current audio runtime state. */
export interface CmdAudioStateGetArgs {
  includeListener?: boolean;
  includeSources?: boolean;
  includeStreams?: boolean;
}

export interface AudioListenerBindingState {
  realmId: number;
  modelId: number;
}

export interface AudioSourceStateEntry {
  sourceId: number;
  realmId?: number;
  modelId?: number;
  position: [number, number, number];
  velocity: [number, number, number];
  orientation: [number, number, number, number];
  gain: number;
  pitch: number;
}

export interface AudioStreamStateEntry {
  resourceId: number;
  receivedBytes: number;
  totalBytes: number;
  complete: boolean;
}

export interface CmdResultAudioStateGet {
  success: boolean;
  message: string;
  listener?: AudioListenerBindingState;
  sources: AudioSourceStateEntry[];
  streams: AudioStreamStateEntry[];
}
