import type { KeyboardEvent } from '../../types/events/keyboard';
import type { PointerEvent } from '../../types/events/pointer';
import type { GamepadEvent } from '../../types/events/gamepad';
import type { SystemEvent } from '../../types/events/system';
import type { UiEvent } from '../../types/events/ui';
import type { WindowEvent } from '../../types/events/window';
import type { System } from '../ecs';
import { applyGamepadEvent } from '../input/core/gamepad';
import { applyKeyboardEvent } from '../input/core/keyboard';
import { applyPointerEvent } from '../input/core/pointer';
import { ensureInputMirrorState, resetInputMirrorFrame } from '../input/core/state';

/**
 * Mirrors inbound core events into world-level ECS state components.
 *
 * This system owns runtime input snapshots for keyboard, pointer, window,
 * gamepad, system, and UI event streams.
 */
export const InputMirrorSystem: System = (world) => {
  const state = ensureInputMirrorState(world);
  resetInputMirrorFrame(state);

  // Process inbound events
  for (let i = 0; i < world.inboundEvents.length; i++) {
    const event = world.inboundEvents[i];
    if (!event) continue;

    // Keyboard events
    if (event.type === 'keyboard') {
      const kbEvent = event.content as KeyboardEvent;
      applyKeyboardEvent(state.inputState, kbEvent);
    }

    // Pointer events
    else if (event.type === 'pointer') {
      const ptrEvent = event.content as PointerEvent;
      applyPointerEvent(state.inputState, ptrEvent);
    }

    // Window events
    else if (event.type === 'window') {
      const winEvent = event.content as WindowEvent;

      if (winEvent.event === 'on-close-request') {
        state.windowState.closeRequested = true;
      } else if (winEvent.event === 'on-focus') {
        if (state.windowState.focused !== winEvent.data.focused) {
          state.windowState.focusChangedThisFrame = true;
        }
        state.windowState.focused = winEvent.data.focused;
      } else if (winEvent.event === 'on-resize') {
        state.windowState.size = [winEvent.data.width, winEvent.data.height];
        state.windowState.resizedThisFrame = true;
      } else if (winEvent.event === 'on-move') {
        state.windowState.position = winEvent.data.position;
        state.windowState.movedThisFrame = true;
      } else if (winEvent.event === 'on-scale-factor-change') {
        state.windowState.scaleFactor = winEvent.data.scaleFactor;
      } else if (winEvent.event === 'on-state-change') {
        state.windowState.lifecycleState = winEvent.data.state;
      } else if (winEvent.event === 'on-pointer-capture-change') {
        state.windowState.pointerCapture = {
          mode: winEvent.data.capture.mode,
          active: winEvent.data.capture.active,
          reason: winEvent.data.capture.reason
        };
      }
    }
    // Gamepad events
    else if (event.type === 'gamepad') {
      const gpEvent = event.content as GamepadEvent;
      applyGamepadEvent(state.gamepadState, gpEvent);
    }
    // System events
    else if (event.type === 'system') {
      const sysEvent = event.content as SystemEvent;
      state.systemEventState.eventsThisFrame.push(sysEvent);
      if (sysEvent.event === 'error') {
        state.systemEventState.lastError = {
          scope: sysEvent.data.scope,
          message: sysEvent.data.message,
          commandId: sysEvent.data.commandId,
          commandType: sysEvent.data.commandType
        };
      }
    }
    // UI events
    else if (event.type === 'ui') {
      state.uiEventState.eventsThisFrame.push(event.content as UiEvent);
    }
  }
  world.inboundEvents.length = 0;
};
