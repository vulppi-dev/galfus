import type { GamepadEvent, SystemEvent, UiEvent } from '../../../types/events';
import type { InputStateComponent, WindowStateComponent } from '../../ecs';
import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import { createVec2Tuple } from '../../math/tuples';
import { WORLD_ENTITY_ID } from './common';

/**
 * Gets the InputState component for a world.
 * Returns undefined if the system hasn't run yet.
 */
function getInputState(worldId: number): InputStateComponent | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('InputState') as InputStateComponent | undefined;
}

/**
 * Gets the WindowState component for a world.
 * Returns undefined if the system hasn't run yet.
 */
function getWindowState(worldId: number): WindowStateComponent | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('WindowState') as WindowStateComponent | undefined;
}

/** Checks if a key is currently pressed. */
export function isKeyPressed(worldId: number, keyCode: number): boolean {
  const state = getInputState(worldId);
  return state?.keysPressed.has(keyCode) ?? false;
}

/** Checks if a key was just pressed this frame. */
export function isKeyJustPressed(worldId: number, keyCode: number): boolean {
  const state = getInputState(worldId);
  return state?.keysJustPressed.has(keyCode) ?? false;
}

/** Checks if a key was just released this frame. */
export function isKeyJustReleased(worldId: number, keyCode: number): boolean {
  const state = getInputState(worldId);
  return state?.keysJustReleased.has(keyCode) ?? false;
}

/** Gets the current pointer position in window space. */
export function getPointerPosition(worldId: number): [number, number] {
  const state = getInputState(worldId);
  return state?.pointerPosition ?? createVec2Tuple();
}

/** Gets the real drawn window area associated with the latest pointer event. */
export function getPointerWindowSize(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.pointerWindowSize ?? null;
}

/** Gets the pointer movement delta for this frame in window space. */
export function getPointerDelta(worldId: number): [number, number] {
  const state = getInputState(worldId);
  return state?.pointerDelta ?? createVec2Tuple();
}

/** Gets the pointer position relative to the current routed target. */
export function getPointerTargetPosition(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.pointerPositionTarget ?? null;
}

/** Gets the real drawn target area associated with the latest pointer event. */
export function getPointerTargetSize(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.pointerTargetSize ?? null;
}

/** Gets the pointer movement delta relative to the current routed target. */
export function getPointerTargetDelta(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.pointerTargetDelta ?? null;
}

/** Gets the routed target under pointer, when available. */
export function getPointerTargetId(worldId: number): number | null {
  const state = getInputState(worldId);
  return state?.pointerTargetId ?? null;
}

/** Gets pointer UV (0..1) in routed target space, when available. */
export function getPointerTargetUv(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.pointerTargetUv ?? null;
}

/** Checks if a pointer button is currently pressed. */
export function isPointerButtonPressed(worldId: number, button: number): boolean {
  const state = getInputState(worldId);
  return state?.pointerButtons.has(button) ?? false;
}

/** Checks if a pointer button was just pressed this frame. */
export function isPointerButtonJustPressed(worldId: number, button: number): boolean {
  const state = getInputState(worldId);
  return state?.pointerJustPressed.has(button) ?? false;
}

/** Gets the scroll delta for this frame. */
export function getScrollDelta(worldId: number): [number, number] {
  const state = getInputState(worldId);
  return state?.scrollDelta ?? createVec2Tuple();
}

/** Returns true while IME composition is active for this world. */
export function isImeEnabled(worldId: number): boolean {
  const state = getInputState(worldId);
  return state?.imeEnabled ?? false;
}

/** Returns current IME preedit text, if any. */
export function getImePreeditText(worldId: number): string | null {
  const state = getInputState(worldId);
  return state?.imePreeditText ?? null;
}

/** Returns current IME cursor range inside preedit text, if available. */
export function getImeCursorRange(worldId: number): [number, number] | null {
  const state = getInputState(worldId);
  return state?.imeCursorRange ?? null;
}

/** Returns last IME committed text for the current frame, if any. */
export function getImeCommitText(worldId: number): string | null {
  const state = getInputState(worldId);
  return state?.imeCommitText ?? null;
}

/** Gets the current window size. */
export function getWindowSize(worldId: number): [number, number] {
  const state = getWindowState(worldId);
  return state?.size ?? createVec2Tuple(800, 600);
}

/** Gets the current window position. */
export function getWindowPosition(worldId: number): [number, number] {
  const state = getWindowState(worldId);
  return state?.position ?? createVec2Tuple();
}

/** Checks if the window is focused. */
export function isWindowFocused(worldId: number): boolean {
  const state = getWindowState(worldId);
  return state?.focused ?? false;
}

/** Checks if a close was requested this frame. */
export function isWindowCloseRequested(worldId: number): boolean {
  const state = getWindowState(worldId);
  return state?.closeRequested ?? false;
}

/** Checks if the window was resized this frame. */
export function wasWindowResized(worldId: number): boolean {
  const state = getWindowState(worldId);
  return state?.resizedThisFrame ?? false;
}

/** Gets the window scale factor (DPI scaling). */
export function getWindowScaleFactor(worldId: number): number {
  const state = getWindowState(worldId);
  return state?.scaleFactor ?? 1.0;
}

/** Gets the latest lifecycle state reported by window events. */
export function getWindowLifecycleState(
  worldId: number
): 'minimized' | 'maximized' | 'windowed' | 'fullscreen' | 'windowed-fullscreen' | null {
  const state = getWindowState(worldId);
  return state?.lifecycleState ?? null;
}

/** Gets the latest pointer-capture snapshot reported by window events. */
export function getWindowPointerCaptureState(
  worldId: number
): { mode: 'none' | 'confined' | 'locked'; active: boolean; reason?: string } | null {
  const state = getWindowState(worldId);
  return state?.pointerCapture ?? null;
}

function getGamepadState(worldId: number):
  | {
      connected: Map<number, { name: string }>;
      buttons: Map<number, Map<number, { pressed: boolean; value: number }>>;
      axes: Map<number, Map<number, number>>;
      eventsThisFrame: GamepadEvent[];
    }
  | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('GamepadState') as
    | {
        connected: Map<number, { name: string }>;
        buttons: Map<number, Map<number, { pressed: boolean; value: number }>>;
        axes: Map<number, Map<number, number>>;
        eventsThisFrame: GamepadEvent[];
      }
    | undefined;
}

function getSystemEventState(worldId: number):
  | {
      eventsThisFrame: SystemEvent[];
      lastError?: {
        scope: string;
        message: string;
        commandId?: number;
        commandType?: string;
      };
    }
  | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('SystemEventState') as
    | {
        eventsThisFrame: SystemEvent[];
        lastError?: {
          scope: string;
          message: string;
          commandId?: number;
          commandType?: string;
        };
      }
    | undefined;
}

function getUiEventState(worldId: number): { eventsThisFrame: UiEvent[] } | undefined {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('UiEventState') as { eventsThisFrame: UiEvent[] } | undefined;
}

export function getGamepadEvents(worldId: number): GamepadEvent[] {
  return getGamepadState(worldId)?.eventsThisFrame ?? [];
}

/** Lists currently connected gamepads sorted by id. */
export function getConnectedGamepads(worldId: number): Array<{ gamepadId: number; name: string }> {
  const connected = getGamepadState(worldId)?.connected;
  if (!connected) return [];
  const out: Array<{ gamepadId: number; name: string }> = [];
  for (const [gamepadId, info] of connected) {
    out.push({ gamepadId, name: info.name });
  }
  out.sort((a, b) => a.gamepadId - b.gamepadId);
  return out;
}

/** Returns current value of a gamepad axis or 0 when unavailable. */
export function getGamepadAxis(worldId: number, gamepadId: number, axis: number): number {
  return getGamepadState(worldId)?.axes.get(gamepadId)?.get(axis) ?? 0;
}

/** Returns whether a gamepad button is currently pressed. */
export function isGamepadButtonPressed(
  worldId: number,
  gamepadId: number,
  button: number
): boolean {
  return getGamepadState(worldId)?.buttons.get(gamepadId)?.get(button)?.pressed ?? false;
}

/** Returns system events mirrored in the current frame. */
export function getSystemEvents(worldId: number): SystemEvent[] {
  return getSystemEventState(worldId)?.eventsThisFrame ?? [];
}

/** Returns the last system error seen by the world, if any. */
export function getLastSystemError(worldId: number): {
  scope: string;
  message: string;
  commandId?: number;
  commandType?: string;
} | null {
  return getSystemEventState(worldId)?.lastError ?? null;
}

/** Returns UI events mirrored in the current frame. */
export function getUiEvents(worldId: number): UiEvent[] {
  return getUiEventState(worldId)?.eventsThisFrame ?? [];
}
