import type {
  CmdInputTargetListenerDisposeArgs,
  CmdInputTargetListenerListArgs,
  CmdInputTargetListenerUpsertArgs
} from '../../types/cmds/input';
import { KeyCode } from '../../types/events/keyboard';
import type { SystemEvent } from '../../types/events/system';
import {
  getConnectedGamepads,
  getGamepadAxis,
  getGamepadEvents,
  getImeCommitText,
  getImeCursorRange,
  getImePreeditText,
  getLastSystemError,
  getPointerDelta,
  getPointerPosition,
  getPointerWindowSize,
  getRoutedPointerSnapshotByWorld,
  getScrollDelta,
  getSystemEvents,
  getWindowLifecycleState,
  getWindowPosition,
  getWindowPointerCaptureState,
  getWindowScaleFactor,
  getWindowSize,
  isGamepadButtonPressed,
  isImeEnabled,
  isKeyPressed,
  isKeyJustPressed,
  isKeyJustReleased,
  isPointerButtonJustPressed,
  isPointerButtonPressed,
  isWindowFocused,
  isWindowCloseRequested,
  wasWindowResized
} from '../input/api';
import {
  disposeInputTargetListener as disposeInputTargetListenerRaw,
  listInputTargetListeners as listInputTargetListenersRaw,
  upsertInputTargetListener as upsertInputTargetListenerRaw
} from './entities';
import type { CommandId, World3DId } from './types';
import { asCommandId, asWorldNumber } from './types';

export { KeyCode };

/** Creates or updates a pointer listener routed by target id. */
export function upsert3DInputTargetListener(
  worldId: World3DId,
  args: CmdInputTargetListenerUpsertArgs
): CommandId {
  return asCommandId(upsertInputTargetListenerRaw(asWorldNumber(worldId), args));
}

/** Disposes a pointer listener routed by target id. */
export function dispose3DInputTargetListener(
  worldId: World3DId,
  args: CmdInputTargetListenerDisposeArgs
): CommandId {
  return asCommandId(disposeInputTargetListenerRaw(asWorldNumber(worldId), args));
}

/** Requests current pointer listener list from core. */
export function list3DInputTargetListeners(
  worldId: World3DId,
  args: CmdInputTargetListenerListArgs = {}
): CommandId {
  return asCommandId(listInputTargetListenersRaw(asWorldNumber(worldId), args));
}

/**
 * Returns system events filtered to input-target-listener-event.
 */
export function get3DTargetPointerEvents(
  worldId: World3DId
): Extract<SystemEvent, { event: 'input-target-listener-event' }>[] {
  const events = getSystemEvents(asWorldNumber(worldId));
  return events.filter(
    (event): event is Extract<SystemEvent, { event: 'input-target-listener-event' }> =>
      event.event === 'input-target-listener-event'
  );
}

/** Returns true while a key is pressed in this world input state. */
export function is3DKeyPressed(worldId: World3DId, keyCode: number): boolean {
  return isKeyPressed(asWorldNumber(worldId), keyCode);
}

/** Returns true when a key was pressed in this frame. */
export function is3DKeyJustPressed(worldId: World3DId, keyCode: number): boolean {
  return isKeyJustPressed(asWorldNumber(worldId), keyCode);
}

/** Returns true when a key was released in this frame. */
export function is3DKeyJustReleased(worldId: World3DId, keyCode: number): boolean {
  return isKeyJustReleased(asWorldNumber(worldId), keyCode);
}

/** Returns true while IME composition is active in this world. */
export function is3DImeEnabled(worldId: World3DId): boolean {
  return isImeEnabled(asWorldNumber(worldId));
}

/** Returns current IME preedit text, if any. */
export function get3DImePreeditText(worldId: World3DId): string | null {
  return getImePreeditText(asWorldNumber(worldId));
}

/** Returns current IME cursor range inside preedit text, if available. */
export function get3DImeCursorRange(worldId: World3DId): vec2 | null {
  return getImeCursorRange(asWorldNumber(worldId));
}

/** Returns last IME committed text for the current frame, if any. */
export function get3DImeCommitText(worldId: World3DId): string | null {
  return getImeCommitText(asWorldNumber(worldId));
}

/** Returns true when close was requested for this world's primary window. */
export function is3DWindowCloseRequested(worldId: World3DId): boolean {
  return isWindowCloseRequested(asWorldNumber(worldId));
}

/** Returns current size for this world's primary window. */
export function get3DWindowSize(worldId: World3DId): vec2 {
  return getWindowSize(asWorldNumber(worldId));
}

/** Returns current position for this world's primary window. */
export function get3DWindowPosition(worldId: World3DId): vec2 {
  return getWindowPosition(asWorldNumber(worldId));
}

/** Returns true while this world's primary window is focused. */
export function is3DWindowFocused(worldId: World3DId): boolean {
  return isWindowFocused(asWorldNumber(worldId));
}

/** Returns true if this world's primary window resized this frame. */
export function was3DWindowResized(worldId: World3DId): boolean {
  return wasWindowResized(asWorldNumber(worldId));
}

/** Returns scale factor (DPI scaling) for this world's primary window. */
export function get3DWindowScaleFactor(worldId: World3DId): number {
  return getWindowScaleFactor(asWorldNumber(worldId));
}

/** Returns latest lifecycle state reported by the window subsystem. */
export function get3DWindowLifecycleState(
  worldId: World3DId
): 'minimized' | 'maximized' | 'windowed' | 'fullscreen' | 'windowed-fullscreen' | null {
  return getWindowLifecycleState(asWorldNumber(worldId));
}

/** Returns latest pointer-capture snapshot reported by the window subsystem. */
export function get3DWindowPointerCaptureState(
  worldId: World3DId
): { mode: 'none' | 'confined' | 'locked'; active: boolean; reason?: string } | null {
  return getWindowPointerCaptureState(asWorldNumber(worldId));
}

/** Returns current pointer position in window space. */
export function get3DPointerPosition(worldId: World3DId): vec2 {
  return getPointerPosition(asWorldNumber(worldId));
}

/** Returns real drawn window size from the latest pointer event, if available. */
export function get3DPointerWindowSize(worldId: World3DId): vec2 | null {
  return getPointerWindowSize(asWorldNumber(worldId));
}

/** Returns pointer delta in window space for the current frame. */
export function get3DPointerDelta(worldId: World3DId): vec2 {
  return getPointerDelta(asWorldNumber(worldId));
}

/** Returns pointer position relative to routed target, if available. */
export function get3DPointerTargetPosition(worldId: World3DId): vec2 | null {
  return getRoutedPointerSnapshotByWorld(asWorldNumber(worldId))?.pointerTargetPosition ?? null;
}

/** Returns real drawn target size from the latest pointer event, if available. */
export function get3DPointerTargetSize(worldId: World3DId): vec2 | null {
  return getRoutedPointerSnapshotByWorld(asWorldNumber(worldId))?.pointerTargetSize ?? null;
}

/** Returns pointer delta relative to routed target for the current frame. */
export function get3DPointerTargetDelta(worldId: World3DId): vec2 | null {
  return getRoutedPointerSnapshotByWorld(asWorldNumber(worldId))?.pointerTargetDelta ?? null;
}

/** Returns routed target id under pointer, when available. */
export function get3DPointerTargetId(worldId: World3DId): number | null {
  return getRoutedPointerSnapshotByWorld(asWorldNumber(worldId))?.pointerTargetId ?? null;
}

/** Returns pointer UV (0..1) in routed target space, when available. */
export function get3DPointerTargetUv(worldId: World3DId): vec2 | null {
  return getRoutedPointerSnapshotByWorld(asWorldNumber(worldId))?.pointerTargetUv ?? null;
}

/** Returns true while a pointer button is pressed in this world. */
export function is3DPointerButtonPressed(worldId: World3DId, button: number): boolean {
  return isPointerButtonPressed(asWorldNumber(worldId), button);
}

/** Returns true when a pointer button was pressed in this frame. */
export function is3DPointerButtonJustPressed(worldId: World3DId, button: number): boolean {
  return isPointerButtonJustPressed(asWorldNumber(worldId), button);
}

/** Returns scroll delta in window space for the current frame. */
export function get3DScrollDelta(worldId: World3DId): vec2 {
  return getScrollDelta(asWorldNumber(worldId));
}

/** Returns gamepad events mirrored in the current frame. */
export function get3DGamepadEvents(worldId: World3DId): ReturnType<typeof getGamepadEvents> {
  return getGamepadEvents(asWorldNumber(worldId));
}

/** Returns connected gamepads sorted by id. */
export function get3DConnectedGamepads(
  worldId: World3DId
): ReturnType<typeof getConnectedGamepads> {
  return getConnectedGamepads(asWorldNumber(worldId));
}

/** Returns current value of a gamepad axis or 0 when unavailable. */
export function get3DGamepadAxis(worldId: World3DId, gamepadId: number, axis: number): number {
  return getGamepadAxis(asWorldNumber(worldId), gamepadId, axis);
}

/** Returns whether a gamepad button is currently pressed. */
export function is3DGamepadButtonPressed(
  worldId: World3DId,
  gamepadId: number,
  button: number
): boolean {
  return isGamepadButtonPressed(asWorldNumber(worldId), gamepadId, button);
}

/** Returns last system error seen by this world, if any. */
export function get3DLastSystemError(worldId: World3DId): ReturnType<typeof getLastSystemError> {
  return getLastSystemError(asWorldNumber(worldId));
}

/** Returns all system events mirrored in the current frame. */
export function get3DSystemEvents(worldId: World3DId): ReturnType<typeof getSystemEvents> {
  return getSystemEvents(asWorldNumber(worldId));
}
import type { vec2 } from 'gl-matrix';
