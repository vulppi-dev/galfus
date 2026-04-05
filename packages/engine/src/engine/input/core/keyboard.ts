import type { KeyboardEvent } from '../../../types/events/keyboard';
import type { InputStateComponent } from '../../ecs/components';

export function applyKeyboardEvent(
  inputState: InputStateComponent,
  keyboardEvent: KeyboardEvent,
): void {
  if (keyboardEvent.event === 'on-input') {
    const keyCode = keyboardEvent.data.keyCode;
    const pressed = keyboardEvent.data.state === 'pressed';

    if (pressed) {
      if (!inputState.keysPressed.has(keyCode)) {
        inputState.keysJustPressed.add(keyCode);
      }
      inputState.keysPressed.add(keyCode);
    } else {
      inputState.keysPressed.delete(keyCode);
      inputState.keysJustReleased.add(keyCode);
    }
    return;
  }

  if (keyboardEvent.event === 'on-ime-enable') {
    inputState.imeEnabled = true;
    return;
  }

  if (keyboardEvent.event === 'on-ime-preedit') {
    inputState.imeEnabled = true;
    inputState.imePreeditText = keyboardEvent.data.text;
    inputState.imeCursorRange = keyboardEvent.data.cursorRange;
    return;
  }

  if (keyboardEvent.event === 'on-ime-commit') {
    inputState.imeEnabled = true;
    inputState.imeCommitText = keyboardEvent.data.text;
    inputState.imePreeditText = undefined;
    inputState.imeCursorRange = undefined;
    return;
  }

  if (keyboardEvent.event === 'on-ime-disable') {
    inputState.imeEnabled = false;
    inputState.imePreeditText = undefined;
    inputState.imeCursorRange = undefined;
  }
}
