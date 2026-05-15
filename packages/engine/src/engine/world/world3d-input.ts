import { KeyCode } from '../../types/events/keyboard';
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
import type { World3DId } from './types';
import { asWorldNumber } from './types';

export { KeyCode };

/** Returns true while a key is pressed in this world input state.
 *
 * @example
 * ```ts
 * const pressed = World3D.is3DKeyPressed(worldId, KeyCode.Space);
 * ```
 */
export function is3DKeyPressed(worldId: World3DId, keyCode: number): boolean {
  return isKeyPressed(asWorldNumber(worldId), keyCode);
}

/** Returns true when a key was pressed in this frame.
 *
 * @example
 * ```ts
 * const justPressed = World3D.is3DKeyJustPressed(worldId, KeyCode.KeyW);
 * ```
 */
export function is3DKeyJustPressed(worldId: World3DId, keyCode: number): boolean {
  return isKeyJustPressed(asWorldNumber(worldId), keyCode);
}

/** Returns true when a key was released in this frame.
 *
 * @example
 * ```ts
 * const justReleased = World3D.is3DKeyJustReleased(worldId, KeyCode.KeyW);
 * ```
 */
export function is3DKeyJustReleased(worldId: World3DId, keyCode: number): boolean {
  return isKeyJustReleased(asWorldNumber(worldId), keyCode);
}

/** Returns true while IME composition is active in this world.
 *
 * @example
 * ```ts
 * const active = World3D.is3DImeEnabled(worldId);
 * ```
 */
export function is3DImeEnabled(worldId: World3DId): boolean {
  return isImeEnabled(asWorldNumber(worldId));
}

/** Returns current IME preedit text, if any.
 *
 * @example
 * ```ts
 * const preedit = World3D.get3DImePreeditText(worldId);
 * ```
 */
export function get3DImePreeditText(worldId: World3DId): string | null {
  return getImePreeditText(asWorldNumber(worldId));
}

/** Returns current IME cursor range inside preedit text, if available.
 *
 * @example
 * ```ts
 * const range = World3D.get3DImeCursorRange(worldId);
 * ```
 */
export function get3DImeCursorRange(worldId: World3DId): vec2 | null {
  return getImeCursorRange(asWorldNumber(worldId));
}

/** Returns last IME committed text for the current frame, if any.
 *
 * @example
 * ```ts
 * const text = World3D.get3DImeCommitText(worldId);
 * ```
 */
export function get3DImeCommitText(worldId: World3DId): string | null {
  return getImeCommitText(asWorldNumber(worldId));
}

/** Returns true when close was requested for this world's primary window.
 *
 * @example
 * ```ts
 * const closing = World3D.is3DWindowCloseRequested(worldId);
 * ```
 */
export function is3DWindowCloseRequested(worldId: World3DId): boolean {
  return isWindowCloseRequested(asWorldNumber(worldId));
}

/** Returns current size for this world's primary window.
 *
 * @example
 * ```ts
 * const size = World3D.get3DWindowSize(worldId);
 * ```
 */
export function get3DWindowSize(worldId: World3DId): vec2 {
  return getWindowSize(asWorldNumber(worldId));
}

/** Returns current position for this world's primary window.
 *
 * @example
 * ```ts
 * const position = World3D.get3DWindowPosition(worldId);
 * ```
 */
export function get3DWindowPosition(worldId: World3DId): vec2 {
  return getWindowPosition(asWorldNumber(worldId));
}

/** Returns true while this world's primary window is focused.
 *
 * @example
 * ```ts
 * const focused = World3D.is3DWindowFocused(worldId);
 * ```
 */
export function is3DWindowFocused(worldId: World3DId): boolean {
  return isWindowFocused(asWorldNumber(worldId));
}

/** Returns true if this world's primary window resized this frame.
 *
 * @example
 * ```ts
 * const resized = World3D.was3DWindowResized(worldId);
 * ```
 */
export function was3DWindowResized(worldId: World3DId): boolean {
  return wasWindowResized(asWorldNumber(worldId));
}

/** Returns scale factor (DPI scaling) for this world's primary window.
 *
 * @example
 * ```ts
 * const scaleFactor = World3D.get3DWindowScaleFactor(worldId);
 * ```
 */
export function get3DWindowScaleFactor(worldId: World3DId): number {
  return getWindowScaleFactor(asWorldNumber(worldId));
}

/** Returns the latest lifecycle state reported by the window subsystem.
 *
 * @example
 * ```ts
 * const state = World3D.get3DWindowLifecycleState(worldId);
 * ```
 */
export function get3DWindowLifecycleState(
  worldId: World3DId
): 'minimized' | 'maximized' | 'windowed' | 'fullscreen' | 'windowed-fullscreen' | null {
  return getWindowLifecycleState(asWorldNumber(worldId));
}

/** Returns the latest pointer-capture snapshot reported by the window subsystem.
 *
 * @example
 * ```ts
 * const capture = World3D.get3DWindowPointerCaptureState(worldId);
 * ```
 */
export function get3DWindowPointerCaptureState(
  worldId: World3DId
): { mode: 'none' | 'confined' | 'locked'; active: boolean; reason?: string } | null {
  return getWindowPointerCaptureState(asWorldNumber(worldId));
}

/** Returns current pointer position in window space.
 *
 * @example
 * ```ts
 * const pointer = World3D.get3DPointerPosition(worldId);
 * ```
 */
export function get3DPointerPosition(worldId: World3DId): vec2 {
  return getPointerPosition(asWorldNumber(worldId));
}

/** Returns real drawn window size from the latest pointer event, if available.
 *
 * @example
 * ```ts
 * const size = World3D.get3DPointerWindowSize(worldId);
 * ```
 */
export function get3DPointerWindowSize(worldId: World3DId): vec2 | null {
  return getPointerWindowSize(asWorldNumber(worldId));
}

/** Returns pointer delta in window space for the current frame.
 *
 * @example
 * ```ts
 * const delta = World3D.get3DPointerDelta(worldId);
 * ```
 */
export function get3DPointerDelta(worldId: World3DId): vec2 {
  return getPointerDelta(asWorldNumber(worldId));
}

/** Returns pointer position relative to a routed target, if available.
 *
 * @example
 * ```ts
 * const position = World3D.get3DPointerTargetPosition(worldId);
 * ```
 */
export function get3DPointerTargetPosition(_worldId: World3DId): vec2 | null {
  return null;
}

export function get3DPointerTargetSize(_worldId: World3DId): vec2 | null {
  return null;
}

export function get3DPointerTargetDelta(_worldId: World3DId): vec2 | null {
  return null;
}

export function get3DPointerTargetId(_worldId: World3DId): number | null {
  return null;
}

export function get3DPointerTargetUv(_worldId: World3DId): vec2 | null {
  return null;
}

/** Returns true while a pointer button is pressed in this world.
 *
 * @example
 * ```ts
 * const down = World3D.is3DPointerButtonPressed(worldId, 0);
 * ```
 */
export function is3DPointerButtonPressed(worldId: World3DId, button: number): boolean {
  return isPointerButtonPressed(asWorldNumber(worldId), button);
}

/** Returns true when a pointer button was pressed in this frame.
 *
 * @example
 * ```ts
 * const clicked = World3D.is3DPointerButtonJustPressed(worldId, 0);
 * ```
 */
export function is3DPointerButtonJustPressed(worldId: World3DId, button: number): boolean {
  return isPointerButtonJustPressed(asWorldNumber(worldId), button);
}

/** Returns scroll delta in window space for the current frame.
 *
 * @example
 * ```ts
 * const scroll = World3D.get3DScrollDelta(worldId);
 * ```
 */
export function get3DScrollDelta(worldId: World3DId): vec2 {
  return getScrollDelta(asWorldNumber(worldId));
}

/** Returns gamepad events mirrored in the current frame.
 *
 * @example
 * ```ts
 * const events = World3D.get3DGamepadEvents(worldId);
 * ```
 */
export function get3DGamepadEvents(worldId: World3DId): ReturnType<typeof getGamepadEvents> {
  return getGamepadEvents(asWorldNumber(worldId));
}

/** Returns connected gamepads sorted by id.
 *
 * @example
 * ```ts
 * const gamepads = World3D.get3DConnectedGamepads(worldId);
 * ```
 */
export function get3DConnectedGamepads(
  worldId: World3DId
): ReturnType<typeof getConnectedGamepads> {
  return getConnectedGamepads(asWorldNumber(worldId));
}

/** Returns the current value of a gamepad axis or `0` when unavailable.
 *
 * @example
 * ```ts
 * const x = World3D.get3DGamepadAxis(worldId, 0, 0);
 * ```
 */
export function get3DGamepadAxis(worldId: World3DId, gamepadId: number, axis: number): number {
  return getGamepadAxis(asWorldNumber(worldId), gamepadId, axis);
}

/** Returns whether a gamepad button is currently pressed.
 *
 * @example
 * ```ts
 * const pressed = World3D.is3DGamepadButtonPressed(worldId, 0, 0);
 * ```
 */
export function is3DGamepadButtonPressed(
  worldId: World3DId,
  gamepadId: number,
  button: number
): boolean {
  return isGamepadButtonPressed(asWorldNumber(worldId), gamepadId, button);
}

/** Returns the last system error seen by this world, if any.
 *
 * @example
 * ```ts
 * const error = World3D.get3DLastSystemError(worldId);
 * ```
 */
export function get3DLastSystemError(worldId: World3DId): ReturnType<typeof getLastSystemError> {
  return getLastSystemError(asWorldNumber(worldId));
}

/** Returns all system events mirrored in the current frame.
 *
 * @example
 * ```ts
 * const events = World3D.get3DSystemEvents(worldId);
 * ```
 */
export function get3DSystemEvents(worldId: World3DId): ReturnType<typeof getSystemEvents> {
  return getSystemEvents(asWorldNumber(worldId));
}
import type { Vec2 as vec2 } from '../../math/index';
