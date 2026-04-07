import type { vec2 } from 'gl-matrix';
import type { PointerEvent } from '../../../types/events/pointer';
import type { InputStateComponent } from '../../ecs/components';

function resolveWindowSize(data: {
  windowWidth?: number;
  windowHeight?: number;
}): vec2 | undefined {
  if (typeof data.windowWidth === 'number' && typeof data.windowHeight === 'number') {
    return [data.windowWidth, data.windowHeight];
  }
  return undefined;
}

function applyPointerPosition(
  inputState: InputStateComponent,
  globalPosition: vec2,
  windowSize?: vec2
): void {
  const oldGlobalPosition = inputState.pointerPosition;
  inputState.pointerDelta = [
    globalPosition[0] - oldGlobalPosition[0],
    globalPosition[1] - oldGlobalPosition[1]
  ];
  inputState.pointerPosition = globalPosition;
  inputState.pointerWindowSize = windowSize;
}

export function applyPointerEvent(
  inputState: InputStateComponent,
  pointerEvent: PointerEvent
): void {
  if (pointerEvent.event === 'on-move') {
    inputState.pointerWindowId = pointerEvent.data.windowId;
    applyPointerPosition(
      inputState,
      pointerEvent.data.position,
      resolveWindowSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-button') {
    const button = pointerEvent.data.button;
    const pressed = pointerEvent.data.state === 'pressed';

    if (pressed) {
      if (!inputState.pointerButtons.has(button)) {
        inputState.pointerJustPressed.add(button);
      }
      inputState.pointerButtons.add(button);
    } else {
      inputState.pointerButtons.delete(button);
      inputState.pointerJustReleased.add(button);
    }

    inputState.pointerWindowId = pointerEvent.data.windowId;
    applyPointerPosition(
      inputState,
      pointerEvent.data.position,
      resolveWindowSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-scroll') {
    inputState.pointerWindowId = pointerEvent.data.windowId;
    inputState.pointerWindowSize = resolveWindowSize(pointerEvent.data);

    const delta = pointerEvent.data.delta;
    if (delta.type === 'line') {
      inputState.scrollDelta = delta.value;
    } else if (delta.type === 'pixel') {
      inputState.scrollDelta = [delta.value[0] / 20, delta.value[1] / 20];
    }
    return;
  }

  if (pointerEvent.event === 'on-touch') {
    inputState.pointerWindowId = pointerEvent.data.windowId;
    applyPointerPosition(
      inputState,
      pointerEvent.data.position,
      resolveWindowSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-leave') {
    inputState.pointerWindowId = pointerEvent.data.windowId;
    inputState.pointerWindowSize = resolveWindowSize(pointerEvent.data);
    return;
  }

  if (pointerEvent.event === 'on-enter') {
    inputState.pointerWindowId = pointerEvent.data.windowId;
    inputState.pointerWindowSize = resolveWindowSize(pointerEvent.data);
  }
}
