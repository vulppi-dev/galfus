import type {
  CmdAudioListenerCreateArgs,
  CmdAudioListenerDisposeArgs,
  CmdAudioListenerGetArgs,
  CmdAudioListenerUpsertArgs,
  CmdAudioListenerUpdateArgs,
  CmdAudioResourceGetArgs,
  CmdAudioResourceListArgs,
  CmdAudioResourceUpsertArgs,
  CmdAudioResourceDisposeArgs,
  CmdAudioSourceGetArgs,
  CmdAudioSourceListArgs,
  CmdAudioSourceTransportArgs,
  CmdAudioSourceCreateArgs,
  CmdAudioSourceDisposeArgs,
  CmdAudioSourceUpsertArgs,
  CmdAudioSourceUpdateArgs,
  CmdAudioStateGetArgs
} from '../../../types/cmds/audio';
import type { CmdPoseUpdateArgs } from '../../../types/cmds/model';
import type { CmdTextureBindTargetArgs } from '../../../types/cmds/texture';
import { enqueueCommand } from '../../bridge/dispatch';

/**
 * Sends an audio listener update command.
 */
export function audioListenerUpdate(worldId: number, args: CmdAudioListenerUpdateArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-listener-upsert', args);
}

/**
 * Binds a texture id to a texture target output.
 */
export function bindTextureToTarget(worldId: number, args: CmdTextureBindTargetArgs): number {
  return enqueueCommand(worldId, 'cmd-texture-bind-target', args);
}

/**
 * Binds the audio listener to a model.
 */
export function audioListenerCreate(worldId: number, args: CmdAudioListenerCreateArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-listener-upsert', args);
}

/**
 * Upserts audio listener params or binding.
 */
export function audioListenerUpsert(worldId: number, args: CmdAudioListenerUpsertArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-listener-upsert', args);
}

/**
 * Disposes the audio listener binding.
 */
export function audioListenerDispose(worldId: number, args: CmdAudioListenerDisposeArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-listener-dispose', args);
}

export function audioListenerGet(worldId: number, args: CmdAudioListenerGetArgs = {}): number {
  return enqueueCommand(worldId, 'cmd-audio-listener-get', args);
}

/**
 * Creates an audio resource from an uploaded buffer.
 */
export function audioResourceCreate(worldId: number, args: CmdAudioResourceUpsertArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-resource-upsert', args);
}

/**
 * Pushes a chunk into a streaming audio resource.
 */
export function audioResourcePush(worldId: number, args: CmdAudioResourceUpsertArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-resource-upsert', args);
}

export function audioResourceGet(worldId: number, args: CmdAudioResourceGetArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-resource-get', args);
}

export function audioResourceList(worldId: number, args: CmdAudioResourceListArgs = {}): number {
  return enqueueCommand(worldId, 'cmd-audio-resource-list', args);
}

/**
 * Disposes an audio resource.
 */
export function audioResourceDispose(worldId: number, args: CmdAudioResourceDisposeArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-resource-dispose', args);
}

/**
 * Creates an audio source bound to a model.
 */
export function audioSourceCreate(worldId: number, args: CmdAudioSourceCreateArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-source-upsert', args);
}

/**
 * Updates an audio source.
 */
export function audioSourceUpdate(worldId: number, args: CmdAudioSourceUpdateArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-source-upsert', args);
}

/**
 * Upserts audio source params or binding.
 */
export function audioSourceUpsert(worldId: number, args: CmdAudioSourceUpsertArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-source-upsert', args);
}

export function audioSourceGet(worldId: number, args: CmdAudioSourceGetArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-source-get', args);
}

export function audioSourceList(worldId: number, args: CmdAudioSourceListArgs = {}): number {
  return enqueueCommand(worldId, 'cmd-audio-source-list', args);
}

/**
 * Starts playback for an audio source.
 */
export function audioSourcePlay(
  worldId: number,
  args: Omit<CmdAudioSourceTransportArgs, 'action'>
): number {
  return enqueueCommand(worldId, 'cmd-audio-source-transport', {
    ...args,
    action: 'play'
  });
}

/**
 * Pauses playback for an audio source.
 */
export function audioSourcePause(
  worldId: number,
  args: Omit<CmdAudioSourceTransportArgs, 'action'>
): number {
  return enqueueCommand(worldId, 'cmd-audio-source-transport', {
    ...args,
    action: 'pause'
  });
}

/**
 * Stops playback for an audio source.
 */
export function audioSourceStop(
  worldId: number,
  args: Omit<CmdAudioSourceTransportArgs, 'action'>
): number {
  return enqueueCommand(worldId, 'cmd-audio-source-transport', {
    ...args,
    action: 'stop'
  });
}

/**
 * Requests a snapshot of audio runtime state.
 */
export function audioStateGet(worldId: number, args: CmdAudioStateGetArgs = {}): number {
  return enqueueCommand(worldId, 'cmd-audio-state-get', args);
}

/**
 * Disposes an audio source.
 */
export function audioSourceDispose(worldId: number, args: CmdAudioSourceDisposeArgs): number {
  return enqueueCommand(worldId, 'cmd-audio-source-dispose', args);
}

/**
 * Updates a model pose (skinning) using an uploaded matrices buffer.
 */
export function poseUpdate(worldId: number, args: CmdPoseUpdateArgs): number {
  return enqueueCommand(worldId, 'cmd-pose-update', args);
}
