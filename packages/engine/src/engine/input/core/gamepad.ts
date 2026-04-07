import type { GamepadEvent } from '../../../types/events/gamepad';
import type { GamepadStateComponent } from '../../ecs/components';

export function applyGamepadEvent(
  gamepadState: GamepadStateComponent,
  gamepadEvent: GamepadEvent
): void {
  gamepadState.eventsThisFrame.push(gamepadEvent);

  if (gamepadEvent.event === 'on-connect') {
    gamepadState.connected.set(gamepadEvent.data.gamepadId, {
      name: gamepadEvent.data.name
    });
    return;
  }

  if (gamepadEvent.event === 'on-disconnect') {
    gamepadState.connected.delete(gamepadEvent.data.gamepadId);
    gamepadState.buttons.delete(gamepadEvent.data.gamepadId);
    gamepadState.axes.delete(gamepadEvent.data.gamepadId);
    return;
  }

  if (gamepadEvent.event === 'on-button') {
    const id = gamepadEvent.data.gamepadId;
    let buttons = gamepadState.buttons.get(id);
    if (!buttons) {
      buttons = new Map();
      gamepadState.buttons.set(id, buttons);
    }
    buttons.set(gamepadEvent.data.button, {
      pressed: gamepadEvent.data.state === 'pressed',
      value: gamepadEvent.data.value
    });
    return;
  }

  if (gamepadEvent.event === 'on-axis') {
    const id = gamepadEvent.data.gamepadId;
    let axes = gamepadState.axes.get(id);
    if (!axes) {
      axes = new Map();
      gamepadState.axes.set(id, axes);
    }
    axes.set(gamepadEvent.data.axis, gamepadEvent.data.value);
  }
}
